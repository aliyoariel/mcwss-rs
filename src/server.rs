//! The websocket server and its event emitter.

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    net::SocketAddr,
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, MutexGuard, RwLock,
    },
    time::Duration,
};

use futures_util::{future::join_all, SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, watch},
    task::JoinHandle,
};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::{
    client::Client,
    consts::ALL_EVENTS,
    error::{Error, Result},
    events::*,
    types::events::*,
};

/// Configuration of an [`Mcwss`] server.
#[derive(Debug, Clone)]
pub struct McwssConfig {
    /// TCP port to listen on (all interfaces).
    pub port: u16,
    /// Extra Minecraft events (e.g. `"BlockBroken"`) to always subscribe to.
    pub events: HashSet<String>,
    /// Unsubscribe from every event on connection before subscribing.
    pub auto_clear_events: bool,
}

impl McwssConfig {
    pub fn new(port: u16) -> Self {
        Self {
            port,
            events: HashSet::new(),
            auto_clear_events: true,
        }
    }

    pub fn events<I, S>(mut self, events: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.events = events.into_iter().map(Into::into).collect();
        self
    }

    pub fn auto_clear_events(mut self, value: bool) -> Self {
        self.auto_clear_events = value;
        self
    }
}

/// Identifies a registered listener, for [`Mcwss::off`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HandlerId(u64);

type Handler = Arc<dyn Fn(&Event) + Send + Sync + 'static>;

struct Entry {
    id: HandlerId,
    once: bool,
    handler: Handler,
}

struct Running {
    shutdown: watch::Sender<bool>,
    task: JoinHandle<()>,
}

struct Inner {
    config: McwssConfig,
    listeners: RwLock<HashMap<EventKind, Vec<Entry>>>,
    next_handler: AtomicU64,
    next_session: AtomicU64,
    clients: Mutex<HashMap<u64, Client>>,
    running: Mutex<Option<Running>>,
}

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// Minecraft Bedrock websocket server: an event emitter plus connection handling.
///
/// Cloning is cheap; all clones share the same server.
#[derive(Clone)]
pub struct Mcwss {
    inner: Arc<Inner>,
}

impl Mcwss {
    pub fn new(config: McwssConfig) -> Self {
        Self {
            inner: Arc::new(Inner {
                config,
                listeners: RwLock::new(HashMap::new()),
                next_handler: AtomicU64::new(1),
                next_session: AtomicU64::new(1),
                clients: Mutex::new(HashMap::new()),
                running: Mutex::new(None),
            }),
        }
    }

    /// Listen to an event.
    pub fn on<F>(&self, kind: EventKind, handler: F) -> HandlerId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.inner.add(kind, false, Arc::new(handler))
    }

    /// Listen to an event once.
    pub fn once<F>(&self, kind: EventKind, handler: F) -> HandlerId
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.inner.add(kind, true, Arc::new(handler))
    }

    /// Cancel one listener.
    pub fn off(&self, id: HandlerId) {
        self.inner.off(id);
    }

    /// Cancel every listener of an event.
    pub fn off_all(&self, kind: EventKind) {
        self.inner
            .listeners
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .remove(&kind);
    }

    /// Emit an event synchronously, calling listeners in registration order.
    pub fn emit(&self, event: Event) {
        self.inner.emit(event);
    }

    /// Emit an event, running every listener on its own task.
    pub async fn parallel(&self, event: Event) {
        let handlers = self.inner.take_handlers(event.kind());
        let tasks = handlers.into_iter().map(|handler| {
            let event = event.clone();
            tokio::spawn(async move { handler(&event) })
        });
        join_all(tasks).await;
    }

    /// Currently connected clients.
    pub fn clients(&self) -> Vec<Client> {
        lock(&self.inner.clients).values().cloned().collect()
    }

    /// Bind the port and start accepting connections. Emits `Ready`.
    pub async fn start(&self) -> Result<SocketAddr> {
        let already = lock(&self.inner.running).is_some();
        if already {
            return Err(Error::AlreadyRunning);
        }
        let listener = TcpListener::bind(("0.0.0.0", self.inner.config.port)).await?;
        let addr = listener.local_addr()?;

        let (tx, rx) = watch::channel(false);
        let inner = self.inner.clone();
        let mut stop_rx = rx.clone();
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    accepted = listener.accept() => match accepted {
                        Ok((stream, peer)) => {
                            tokio::spawn(handle_connection(inner.clone(), stream, peer, rx.clone()));
                        }
                        Err(e) => {
                            inner.emit(Event::Error(ErrorData::new(None, "IoError", e)));
                            tokio::time::sleep(Duration::from_millis(50)).await;
                        }
                    },
                    _ = stop_rx.changed() => break,
                }
            }
        });

        *lock(&self.inner.running) = Some(Running { shutdown: tx, task });
        self.inner.emit(Event::Ready(ReadyData { addr }));
        Ok(addr)
    }

    /// Stop the server and close every connection. Emits `Close` and `Dispose`.
    pub async fn stop(&self) {
        let running = lock(&self.inner.running).take();
        let Some(running) = running else {
            return;
        };
        let _ = running.shutdown.send(true);
        let _ = running.task.await;
        self.inner.emit(Event::Close(CloseData {
            client: None,
            code: None,
        }));
        self.inner.emit(Event::Dispose);
    }
}

impl Inner {
    fn add(&self, kind: EventKind, once: bool, handler: Handler) -> HandlerId {
        let id = HandlerId(self.next_handler.fetch_add(1, Ordering::Relaxed));
        self.listeners
            .write()
            .unwrap_or_else(|e| e.into_inner())
            .entry(kind)
            .or_default()
            .push(Entry { id, once, handler });
        id
    }

    fn off(&self, id: HandlerId) {
        let mut map = self.listeners.write().unwrap_or_else(|e| e.into_inner());
        for entries in map.values_mut() {
            entries.retain(|e| e.id != id);
        }
    }

    /// Clones the handlers of `kind`; `once` handlers are removed.
    fn take_handlers(&self, kind: EventKind) -> Vec<Handler> {
        let snapshot: Vec<(HandlerId, bool, Handler)> = {
            let map = self.listeners.read().unwrap_or_else(|e| e.into_inner());
            map.get(&kind)
                .map(|v| {
                    v.iter()
                        .map(|e| (e.id, e.once, e.handler.clone()))
                        .collect()
                })
                .unwrap_or_default()
        };
        snapshot
            .into_iter()
            .map(|(id, once, handler)| {
                if once {
                    self.off(id);
                }
                handler
            })
            .collect()
    }

    fn emit(&self, event: Event) {
        for handler in self.take_handlers(event.kind()) {
            if catch_unwind(AssertUnwindSafe(|| handler(&event))).is_err() {
                eprintln!("[mcwss] an event listener panicked ({:?})", event.kind());
            }
        }
    }

    /// Subscribe a fresh connection to every event somebody listens to.
    fn auto_subscribe(&self, client: &Client) {
        if self.config.auto_clear_events {
            for name in ALL_EVENTS {
                let _ = client.unsubscribe(name);
            }
        }
        let mut events: BTreeSet<String> = self.config.events.iter().cloned().collect();
        {
            let map = self.listeners.read().unwrap_or_else(|e| e.into_inner());
            for (kind, entries) in map.iter() {
                if entries.is_empty() {
                    continue;
                }
                if let Some(name) = kind.minecraft_name() {
                    events.insert(name.to_string());
                }
            }
        }
        for name in events {
            let _ = client.subscribe(&name);
        }
    }

    fn on_message(&self, raw: String, client: &Client) {
        self.handle_raw(&raw, client);
        self.emit(Event::Message(MessageData {
            raw,
            client: client.clone(),
        }));
    }

    /// Turn a raw packet into a typed event.
    fn handle_raw(&self, raw: &str, client: &Client) {
        let value: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(e) => {
                self.emit(Event::Error(ErrorData::new(
                    Some(client.clone()),
                    "SyntaxError",
                    e,
                )));
                return;
            }
        };

        let purpose = value
            .get("header")
            .and_then(|h| h.get("messagePurpose"))
            .and_then(Value::as_str);
        if purpose == Some("commandResponse") {
            match CommandResponsePacket::deserialize(&value) {
                Ok(p) => self.emit(Event::CommandResponse(CommandResponseData {
                    header: p.header,
                    body: p.body,
                    client: client.clone(),
                })),
                Err(e) => self.emit(Event::Error(ErrorData::new(
                    Some(client.clone()),
                    "Error",
                    e,
                ))),
            }
            return;
        }

        let event = event_name(&value).and_then(|name| match name {
            "BlockBroken" => parse(&value, client).map(Event::BlockBroken),
            "BlockPlaced" => parse(&value, client).map(Event::BlockPlaced),
            "EndOfDay" => parse(&value, client).map(Event::EndOfDay),
            "EntitySpawned" => parse(&value, client).map(Event::EntitySpawned),
            "ItemAcquired" => parse(&value, client).map(Event::ItemAcquired),
            "ItemCrafted" => parse(&value, client).map(Event::ItemCrafted),
            "ItemDestroyed" => parse(&value, client).map(Event::ItemDestroyed),
            "ItemSmelted" => parse(&value, client).map(Event::ItemSmelted),
            "ItemUsed" => parse(&value, client).map(Event::ItemUsed),
            "JukeboxUsed" => parse(&value, client).map(Event::JukeboxUsed),
            "MobInteracted" => parse(&value, client).map(Event::MobInteracted),
            "MobKilled" => parse(&value, client).map(Event::MobKilled),
            "PlayerBounced" => parse(&value, client).map(Event::PlayerBounced),
            "PlayerDied" => parse(&value, client).map(Event::PlayerDied),
            "PlayerMessage" => parse(&value, client).map(Event::PlayerMessage),
            "PlayerTeleported" => parse(&value, client).map(Event::PlayerTeleported),
            "PlayerTransform" => parse(&value, client).map(Event::PlayerTransform),
            "PlayerTravelled" => parse(&value, client).map(Event::PlayerTravelled),
            _ => None,
        });

        let event = event.unwrap_or_else(|| {
            Event::UnknownMinecraftEvent(UnknownData {
                raw: value,
                client: client.clone(),
            })
        });
        self.emit(event);
    }
}

/// Validates the base shape of a Minecraft event packet and returns its event name.
fn event_name(value: &Value) -> Option<&str> {
    let header = value.get("header")?.as_object()?;
    if !value.get("body")?.is_object() {
        return None;
    }
    if !header.get("messagePurpose")?.is_string() || !header.get("version")?.is_number() {
        return None;
    }
    let name = header.get("eventName")?.as_str()?;
    ALL_EVENTS.contains(&name).then_some(name)
}

fn parse<B: DeserializeOwned>(value: &Value, client: &Client) -> Option<Data<B>> {
    let packet = Packet::<B>::deserialize(value).ok()?;
    Some(Data {
        header: packet.header,
        body: packet.body,
        client: client.clone(),
    })
}

async fn handle_connection(
    inner: Arc<Inner>,
    stream: TcpStream,
    peer: SocketAddr,
    mut stop_rx: watch::Receiver<bool>,
) {
    let ws = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            inner.emit(Event::Error(ErrorData::new(None, "HandshakeError", e)));
            return;
        }
    };
    let (mut sink, mut stream) = ws.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let id = inner.next_session.fetch_add(1, Ordering::Relaxed);
    let client = Client::new(id, Some(peer), tx);
    lock(&inner.clients).insert(id, client.clone());

    // Writer: everything sent through `Client` is forwarded to the socket.
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let closing = matches!(msg, Message::Close(_));
            if sink.send(msg).await.is_err() || closing {
                break;
            }
        }
        let _ = sink.close().await;
    });

    inner.auto_subscribe(&client);
    inner.emit(Event::Connection(ConnectionData {
        client: client.clone(),
    }));

    let mut code: u16 = 1006;
    loop {
        tokio::select! {
            frame = stream.next() => match frame {
                Some(Ok(Message::Text(text))) => inner.on_message(text.to_string(), &client),
                Some(Ok(Message::Binary(bin))) => {
                    inner.on_message(String::from_utf8_lossy(&bin).into_owned(), &client)
                }
                Some(Ok(Message::Close(frame))) => {
                    code = frame.map(|f| u16::from(f.code)).unwrap_or(1005);
                    break;
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => {
                    inner.emit(Event::Error(ErrorData::new(
                        Some(client.clone()),
                        "WebSocketError",
                        e,
                    )));
                    break;
                }
                None => break,
            },
            _ = stop_rx.changed() => {
                let _ = client.close();
                break;
            }
        }
    }

    lock(&inner.clients).remove(&id);
    inner.emit(Event::Close(CloseData {
        client: Some(client),
        code: Some(code),
    }));
}
