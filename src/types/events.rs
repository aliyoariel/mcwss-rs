//! Typed payloads of the Minecraft events mcwss understands.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use super::minecraft::{lenient_string, BlockObject, EntityObject, ItemObject, Position};
use crate::client::Client;

/// Header of a normal Minecraft event packet.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Header {
    pub event_name: String,
    pub message_purpose: String,
    pub version: i64,
}

/// A Minecraft event: header, typed body and the client that sent it.
#[derive(Debug, Clone)]
pub struct Data<B> {
    pub header: Header,
    pub body: B,
    pub client: Client,
}

/// Wire shape used while deserializing (the client is attached afterwards).
#[derive(Deserialize)]
pub(crate) struct Packet<B> {
    pub header: Header,
    pub body: B,
}

/// An event whose name or shape mcwss does not model, kept as raw JSON.
#[derive(Debug, Clone)]
pub struct UnknownData {
    pub raw: Value,
    pub client: Client,
}

// ---------------------------------------------------------------------------
// Event bodies
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BlockBrokenBody {
    pub block: BlockObject,
    pub count: i64,
    pub destruction_method: i64,
    pub player: EntityObject,
    pub tool: ItemObject,
    pub variant: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct BlockPlacedBody {
    pub block: BlockObject,
    pub count: i64,
    pub placed_under_water: bool,
    pub placement_method: i64,
    pub player: EntityObject,
    pub tool: ItemObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct EndOfDayBody {
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MobType {
    #[serde(rename = "type")]
    pub kind: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EntitySpawnedBody {
    pub mob: MobType,
    pub player: EntityObject,
    pub spawn_type: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemAcquiredBody {
    pub acquisition_method_id: i64,
    pub count: i64,
    pub item: BlockObject,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemCraftedBody {
    pub count: i64,
    pub crafted_automatically: bool,
    pub ending_tab_id: i64,
    pub has_craftable_filter_on: bool,
    pub item: ItemObject,
    pub number_of_tabs_changed: i64,
    pub player: EntityObject,
    pub recipe_book_shown: bool,
    pub starting_tab_id: i64,
    pub used_crafting_table: bool,
    pub used_search_bar: bool,
}

/// Same shape as [`ItemCraftedBody`].
pub type ItemDestroyedBody = ItemCraftedBody;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemSmeltedBody {
    pub fuel_source: BlockObject,
    pub item: BlockObject,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemUsedBody {
    pub count: i64,
    pub item: BlockObject,
    pub player: EntityObject,
    pub use_method: i64,
}

/// Same shape as [`ItemUsedBody`].
pub type JukeboxUsedBody = ItemUsedBody;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct MobInfo {
    pub color: i64,
    #[serde(rename = "type")]
    pub kind: i64,
    pub variant: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MobInteractedBody {
    pub interaction_type: i64,
    pub mob: MobInfo,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MobKilledBody {
    pub armor_feet: ItemObject,
    pub armor_head: ItemObject,
    pub armor_legs: ItemObject,
    pub armor_torso: ItemObject,
    pub is_monster: bool,
    pub kill_method_type: i64,
    pub player: EntityObject,
    pub player_is_hidden_from: bool,
    pub victim: EntityObject,
    pub weapon: ItemObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerBouncedBody {
    pub block: BlockObject,
    pub bounce_height: f64,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Killer {
    pub color: i64,
    pub id: i64,
    #[serde(rename = "type")]
    pub kind: i64,
    pub variant: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerDiedBody {
    pub cause: i64,
    pub in_raid: bool,
    pub killer: Killer,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlayerMessageBody {
    #[serde(deserialize_with = "lenient_string")]
    pub message: String,
    #[serde(deserialize_with = "lenient_string")]
    pub receiver: String,
    #[serde(deserialize_with = "lenient_string")]
    pub sender: String,
    /// `chat`, `say`, `title` or `tell`.
    #[serde(rename = "type", deserialize_with = "lenient_string")]
    pub kind: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerTeleportedBody {
    pub cause: i64,
    pub item_type: i64,
    pub meters_travelled: f64,
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct PlayerTransformBody {
    pub player: EntityObject,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct PlayerTravelledBody {
    pub is_underwater: bool,
    pub meters_travelled: f64,
    pub new_biome: i64,
    pub player: EntityObject,
    pub travel_method: i64,
}

// ---------------------------------------------------------------------------
// Command responses
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CommandResponseHeader {
    pub message_purpose: String,
    pub request_id: String,
    pub version: i64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CommandResponseBody {
    pub status_code: i64,
    pub status_message: Option<String>,
    pub destination: Option<Position>,
    pub victim: Option<Vec<String>>,
    pub current_player_count: Option<i64>,
    pub max_player_count: Option<i64>,
    pub players: Option<String>,
    /// Any other field the command returned.
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Deserialize)]
pub(crate) struct CommandResponsePacket {
    pub header: CommandResponseHeader,
    pub body: CommandResponseBody,
}

/// Response to a command previously sent with [`Client::run`].
#[derive(Debug, Clone)]
pub struct CommandResponseData {
    pub header: CommandResponseHeader,
    pub body: CommandResponseBody,
    pub client: Client,
}
