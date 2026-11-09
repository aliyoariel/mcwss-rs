//! Shared Minecraft object shapes found inside event bodies.
//!
//! Every struct is deserialized leniently: missing fields fall back to their
//! defaults, and string-ish fields accept numbers/booleans too.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Accepts a string, number, bool or null and always yields a `String`.
pub(crate) fn lenient_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match Value::deserialize(deserializer)? {
        Value::String(s) => s,
        Value::Null => String::new(),
        other => other.to_string(),
    })
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl fmt::Display for Position {
    /// Formats as `x,y,z`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{},{},{}", self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Enchantment {
    pub level: i64,
    #[serde(deserialize_with = "lenient_string")]
    pub name: String,
    #[serde(rename = "type")]
    pub kind: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct EntityObject {
    #[serde(deserialize_with = "lenient_string")]
    pub color: String,
    pub dimension: i64,
    /// Usually `minecraft:<name>`.
    #[serde(deserialize_with = "lenient_string")]
    pub id: String,
    #[serde(deserialize_with = "lenient_string")]
    pub name: String,
    pub position: Position,
    #[serde(rename = "type", deserialize_with = "lenient_string")]
    pub kind: String,
    pub variant: i64,
    pub y_rot: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ItemObject {
    pub aux: i64,
    pub enchantments: Vec<Enchantment>,
    pub free_stack_size: i64,
    #[serde(deserialize_with = "lenient_string")]
    pub id: String,
    pub max_stack_size: i64,
    #[serde(deserialize_with = "lenient_string")]
    pub namespace: String,
    pub stack_size: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BlockObject {
    pub aux: i64,
    #[serde(deserialize_with = "lenient_string")]
    pub id: String,
    #[serde(deserialize_with = "lenient_string")]
    pub namespace: String,
}
