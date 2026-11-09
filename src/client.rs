//! A connected Minecraft client and the packets sent to it.

use std::{fmt, net::SocketAddr, path::Path, sync::Arc};

use chrono::{DateTime, Utc};
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::error::{Error, Result};

/// Minecraft formatting colors used by [`Client::chatf`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextColor {
    Green,
    Red,
    Blue,
    Yellow,
}

impl TextColor {
    pub const fn code(self) -> &'static str {
        match self {
            TextColor::Green => "§a",
            TextColor::Red => "§c",
            TextColor::Blue => "§b",
            TextColor::Yellow => "§e",
        }
    }
}

/// Target selectors used by [`Client::chatf`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandTarget {
    All,
    #[default]
    SelfPlayer,
    Random,
    Every,
    Near,
}

impl CommandTarget {
    pub const fn selector(self) -> &'static str {
        match self {
            CommandTarget::All => "@a",
            CommandTarget::SelfPlayer => "@s",
            CommandTarget::Random => "@r",
            CommandTarget::Every => "@e",
            CommandTarget::Near => "@p",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketHeader {
    pub request_id: String,
    pub message_purpose: &'static str,
    pub version: u32,
    pub message_type: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeBody {
    pub event_name: String,
}

#[derive(Debug, Serialize)]
pub struct CommandOrigin {
    #[serde(rename = "type")]
    pub kind: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandBody {
    pub origin: CommandOrigin,
    pub command_line: String,
    pub version: u32,
}

#[derive(Debug, Serialize)]
pub struct SubscribePacket {
    pub body: SubscribeBody,
    pub header: PacketHeader,
}

#[derive(Debug, Serialize)]
pub struct CommandPacket {
    pub body: CommandBody,
    pub header: PacketHeader,
}

/// A packet that can be sent to a client.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum SendPacket {
    Subscribe(SubscribePacket),
    Command(CommandPacket),
}

struct ClientInner {
    session_id: u64,
    session_date: DateTime<Utc>,
    remote_addr: Option<SocketAddr>,
    tx: UnboundedSender<Message>,
}

/// Handle to one connected client. Cheap to clone.
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("session_id", &self.inner.session_id)
            .field("session_date", &self.inner.session_date)
            .field("remote_addr", &self.inner.remote_addr)
            .finish()
    }
}

impl Client {
    pub(crate) fn new(
        session_id: u64,
        remote_addr: Option<SocketAddr>,
        tx: UnboundedSender<Message>,
    ) -> Self {
        Self {
            inner: Arc::new(ClientInner {
                session_id,
                session_date: Utc::now(),
                remote_addr,
                tx,
            }),
        }
    }

    /// Connection identifier, unique within one `Mcwss` instance.
    pub fn session_id(&self) -> u64 {
        self.inner.session_id
    }

    /// Time the connection was established.
    pub fn session_date(&self) -> DateTime<Utc> {
        self.inner.session_date
    }

    /// Remote socket address of the client.
    pub fn remote_addr(&self) -> Option<SocketAddr> {
        self.inner.remote_addr
    }

    /// Send a packet to the client.
    pub fn send(&self, packet: &SendPacket) -> Result<()> {
        let text = serde_json::to_string(packet)?;
        self.inner
            .tx
            .send(Message::Text(text.into()))
            .map_err(|_| Error::ConnectionClosed)
    }

    /// Close the connection.
    pub fn close(&self) -> Result<()> {
        self.inner
            .tx
            .send(Message::Close(None))
            .map_err(|_| Error::ConnectionClosed)
    }

    fn header(purpose: &'static str) -> PacketHeader {
        PacketHeader {
            request_id: Uuid::new_v4().to_string(),
            message_purpose: purpose,
            version: 1,
            message_type: "commandRequest",
        }
    }

    /// Subscribe to a Minecraft event, e.g. `"BlockBroken"`.
    pub fn subscribe(&self, event: &str) -> Result<()> {
        self.send(&SendPacket::Subscribe(SubscribePacket {
            body: SubscribeBody {
                event_name: event.to_string(),
            },
            header: Self::header("subscribe"),
        }))
    }

    /// Unsubscribe from a Minecraft event.
    pub fn unsubscribe(&self, event: &str) -> Result<()> {
        self.send(&SendPacket::Subscribe(SubscribePacket {
            body: SubscribeBody {
                event_name: event.to_string(),
            },
            header: Self::header("unsubscribe"),
        }))
    }

    /// Run one command on the client, e.g. `"say hi"` or `"/say hi"`.
    pub fn run(&self, cmd: impl AsRef<str>) -> Result<()> {
        self.send(&SendPacket::Command(CommandPacket {
            body: CommandBody {
                origin: CommandOrigin { kind: "player" },
                command_line: cmd.as_ref().to_string(),
                version: 1,
            },
            header: Self::header("commandRequest"),
        }))
    }

    /// Run a command built from parts joined by spaces.
    pub fn run_parts<I, S>(&self, parts: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let cmd = parts
            .into_iter()
            .map(|p| p.as_ref().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        self.run(cmd)
    }

    /// Send a normal message to the client (based on `/say`).
    pub fn chat(&self, message: impl AsRef<str>) -> Result<()> {
        self.run_parts(["say", message.as_ref()])
    }

    /// Send an advanced message to the client (based on `/tellraw`).
    pub fn chatf(
        &self,
        message: impl AsRef<str>,
        color: Option<TextColor>,
        sender: Option<&str>,
        target: CommandTarget,
    ) -> Result<()> {
        let mut text = match color {
            Some(c) => format!("{}{}", c.code(), message.as_ref()),
            None => message.as_ref().to_string(),
        };
        if let Some(sender) = sender.filter(|s| !s.is_empty()) {
            text = format!("<{sender}>{text}");
        }
        let raw = serde_json::json!({ "rawtext": [{ "text": text }] }).to_string();
        self.run_parts(["tellraw", target.selector(), raw.as_str()])
    }

    /// Shortcut for `chatf(message, None, None, CommandTarget::SelfPlayer)`.
    pub fn tell(&self, message: impl AsRef<str>) -> Result<()> {
        self.chatf(message, None, None, CommandTarget::SelfPlayer)
    }

    /// Run every line of a `.mcfunction` file on the client.
    ///
    /// Returns `Ok(false)` when the file does not exist.
    pub fn func(&self, file: impl AsRef<Path>) -> Result<bool> {
        let path = file.as_ref();
        if !path.exists() {
            return Ok(false);
        }
        let content = std::fs::read_to_string(path)?;
        for line in content.split('\n') {
            let cmd = line.trim_end_matches('\r');
            if cmd.trim().is_empty() {
                continue;
            }
            self.run(cmd)?;
        }
        Ok(true)
    }
}
