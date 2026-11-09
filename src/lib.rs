//! # mcwss
//!
//! A simple Minecraft Bedrock Edition websocket server framework.
//!
//! [`Mcwss`] is an event emitter: register listeners with [`Mcwss::on`], call
//! [`Mcwss::start`], and use the [`Client`] attached to every event to talk
//! back to the game.
//!
//! ```no_run
//! use mcwss::{Event, EventKind, Mcwss, McwssConfig};
//!
//! #[tokio::main]
//! async fn main() -> mcwss::Result<()> {
//!     let mcwss = Mcwss::new(McwssConfig::new(2333));
//!
//!     mcwss.on(EventKind::BlockBroken, |event| {
//!         if let Event::BlockBroken(data) = event {
//!             println!("{} broke {}", data.body.player.name, data.body.block.id);
//!         }
//!     });
//!
//!     mcwss.start().await?;
//!     tokio::signal::ctrl_c().await?;
//!     mcwss.stop().await;
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod consts;
pub mod error;
pub mod events;
mod server;
pub mod types;

pub use client::{Client, CommandTarget, SendPacket, TextColor};
pub use error::{Error, Result};
pub use events::{CloseData, ConnectionData, ErrorData, Event, EventKind, MessageData, ReadyData};
pub use server::{HandlerId, Mcwss, McwssConfig};
pub use types::events::*;
pub use types::minecraft::{BlockObject, Enchantment, EntityObject, ItemObject, Position};
