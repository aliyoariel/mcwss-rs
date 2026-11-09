//! The event enum emitted by [`crate::Mcwss`].

use std::{fmt::Display, net::SocketAddr};

use crate::{client::Client, types::events::*};

#[derive(Debug, Clone)]
pub struct ReadyData {
    pub addr: SocketAddr,
}

#[derive(Debug, Clone)]
pub struct ErrorData {
    /// `None` for server-level errors.
    pub client: Option<Client>,
    pub name: String,
    pub message: String,
}

impl ErrorData {
    pub fn new(client: Option<Client>, name: &str, message: impl Display) -> Self {
        Self {
            client,
            name: name.to_string(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionData {
    pub client: Client,
}

#[derive(Debug, Clone)]
pub struct MessageData {
    pub raw: String,
    pub client: Client,
}

#[derive(Debug, Clone)]
pub struct CloseData {
    /// `None` when the whole server closes, `Some` when one client closes.
    pub client: Option<Client>,
    /// WebSocket close code of the client (1006 when it vanished abruptly).
    pub code: Option<u16>,
}

/// Everything `Mcwss` can emit: lifecycle events and Minecraft events.
#[derive(Debug, Clone)]
pub enum Event {
    // lifecycle
    Ready(ReadyData),
    Dispose,
    Error(ErrorData),
    Connection(ConnectionData),
    Message(MessageData),
    Close(CloseData),
    // minecraft
    BlockBroken(Data<BlockBrokenBody>),
    BlockPlaced(Data<BlockPlacedBody>),
    EndOfDay(Data<EndOfDayBody>),
    EntitySpawned(Data<EntitySpawnedBody>),
    ItemAcquired(Data<ItemAcquiredBody>),
    ItemCrafted(Data<ItemCraftedBody>),
    ItemDestroyed(Data<ItemDestroyedBody>),
    ItemSmelted(Data<ItemSmeltedBody>),
    ItemUsed(Data<ItemUsedBody>),
    JukeboxUsed(Data<JukeboxUsedBody>),
    MobInteracted(Data<MobInteractedBody>),
    MobKilled(Data<MobKilledBody>),
    PlayerBounced(Data<PlayerBouncedBody>),
    PlayerDied(Data<PlayerDiedBody>),
    PlayerMessage(Data<PlayerMessageBody>),
    PlayerTeleported(Data<PlayerTeleportedBody>),
    PlayerTransform(Data<PlayerTransformBody>),
    PlayerTravelled(Data<PlayerTravelledBody>),
    UnknownMinecraftEvent(UnknownData),
    CommandResponse(CommandResponseData),
}

/// Identifies an event type when registering listeners.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Ready,
    Dispose,
    Error,
    Connection,
    Message,
    Close,
    BlockBroken,
    BlockPlaced,
    EndOfDay,
    EntitySpawned,
    ItemAcquired,
    ItemCrafted,
    ItemDestroyed,
    ItemSmelted,
    ItemUsed,
    JukeboxUsed,
    MobInteracted,
    MobKilled,
    PlayerBounced,
    PlayerDied,
    PlayerMessage,
    PlayerTeleported,
    PlayerTransform,
    PlayerTravelled,
    UnknownMinecraftEvent,
    CommandResponse,
}

impl Event {
    pub fn kind(&self) -> EventKind {
        match self {
            Event::Ready(_) => EventKind::Ready,
            Event::Dispose => EventKind::Dispose,
            Event::Error(_) => EventKind::Error,
            Event::Connection(_) => EventKind::Connection,
            Event::Message(_) => EventKind::Message,
            Event::Close(_) => EventKind::Close,
            Event::BlockBroken(_) => EventKind::BlockBroken,
            Event::BlockPlaced(_) => EventKind::BlockPlaced,
            Event::EndOfDay(_) => EventKind::EndOfDay,
            Event::EntitySpawned(_) => EventKind::EntitySpawned,
            Event::ItemAcquired(_) => EventKind::ItemAcquired,
            Event::ItemCrafted(_) => EventKind::ItemCrafted,
            Event::ItemDestroyed(_) => EventKind::ItemDestroyed,
            Event::ItemSmelted(_) => EventKind::ItemSmelted,
            Event::ItemUsed(_) => EventKind::ItemUsed,
            Event::JukeboxUsed(_) => EventKind::JukeboxUsed,
            Event::MobInteracted(_) => EventKind::MobInteracted,
            Event::MobKilled(_) => EventKind::MobKilled,
            Event::PlayerBounced(_) => EventKind::PlayerBounced,
            Event::PlayerDied(_) => EventKind::PlayerDied,
            Event::PlayerMessage(_) => EventKind::PlayerMessage,
            Event::PlayerTeleported(_) => EventKind::PlayerTeleported,
            Event::PlayerTransform(_) => EventKind::PlayerTransform,
            Event::PlayerTravelled(_) => EventKind::PlayerTravelled,
            Event::UnknownMinecraftEvent(_) => EventKind::UnknownMinecraftEvent,
            Event::CommandResponse(_) => EventKind::CommandResponse,
        }
    }
}

impl EventKind {
    /// The Minecraft event name to subscribe to for this kind, if any.
    pub fn minecraft_name(self) -> Option<&'static str> {
        Some(match self {
            EventKind::BlockBroken => "BlockBroken",
            EventKind::BlockPlaced => "BlockPlaced",
            EventKind::EndOfDay => "EndOfDay",
            EventKind::EntitySpawned => "EntitySpawned",
            EventKind::ItemAcquired => "ItemAcquired",
            EventKind::ItemCrafted => "ItemCrafted",
            EventKind::ItemDestroyed => "ItemDestroyed",
            EventKind::ItemSmelted => "ItemSmelted",
            EventKind::ItemUsed => "ItemUsed",
            EventKind::JukeboxUsed => "JukeboxUsed",
            EventKind::MobInteracted => "MobInteracted",
            EventKind::MobKilled => "MobKilled",
            EventKind::PlayerBounced => "PlayerBounced",
            EventKind::PlayerDied => "PlayerDied",
            EventKind::PlayerMessage => "PlayerMessage",
            EventKind::PlayerTeleported => "PlayerTeleported",
            EventKind::PlayerTransform => "PlayerTransform",
            EventKind::PlayerTravelled => "PlayerTravelled",
            _ => return None,
        })
    }
}
