//! The `mcwss` command line program: logs every Minecraft event and answers
//! the built-in `*/` commands typed in the game chat.

use std::sync::Arc;

use mcwss::{CommandTarget, Event, EventKind, Mcwss, McwssConfig, TextColor};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::{
    i18n::I18n,
    logger::{Level, Logger},
};

struct Ctx {
    logger: Logger,
    i18n: I18n,
}

/// `tr!(ctx, "key", a, b, ...)` -> translated template with `{0}`, `{1}`, ... filled in.
macro_rules! tr {
    ($ctx:expr, $key:expr $(, $arg:expr)* $(,)?) => {
        $ctx.i18n.format($key, &[$(($arg).to_string()),*])
    };
}

/// `on!(mcwss, ctx, Kind, |data| { ... })` registers a listener for one event kind.
macro_rules! on {
    ($m:expr, $ctx:ident, $kind:ident, |$d:ident| $body:block) => {{
        let $ctx = $ctx.clone();
        $m.on(EventKind::$kind, move |event| {
            if let Event::$kind($d) = event $body
        });
    }};
}

fn item_name(namespace: &str, id: &str) -> String {
    if id.is_empty() {
        String::new()
    } else {
        format!("{namespace}:{id}")
    }
}

pub async fn run(port: u16, level: Level, lang: &str) -> Result<(), mcwss::Error> {
    let mcwss = Mcwss::new(McwssConfig::new(port));
    let ctx = Arc::new(Ctx {
        logger: Logger::new(
            level,
            std::env::current_dir().unwrap_or_default().join("logs"),
        ),
        i18n: I18n::new(lang),
    });

    // ---- server events -----------------------------------------------------
    on!(mcwss, ctx, Ready, |d| {
        ctx.logger.info(&format!(
            "WebSocketServer started at ws://localhost:{} ",
            d.addr.port()
        ));
    });

    {
        let ctx = ctx.clone();
        mcwss.on(EventKind::Dispose, move |_| {
            ctx.logger.info("WebSocketServer stopped");
        });
    }

    on!(mcwss, ctx, Error, |d| {
        let who = match &d.client {
            Some(c) => format!("[Client:{}]", c.session_id()),
            None => "[Server]".to_string(),
        };
        ctx.logger.error(&format!("{who} {} {}", d.name, d.message));
    });

    on!(mcwss, ctx, Connection, |d| {
        let ip = d
            .client
            .remote_addr()
            .map(|a| a.ip().to_string())
            .unwrap_or_default();
        ctx.logger.info(&format!(
            "[Client:{}] new connection from {ip}",
            d.client.session_id()
        ));
    });

    on!(mcwss, ctx, Close, |d| {
        match (&d.client, d.code) {
            (Some(client), Some(code)) => ctx.logger.info(&format!(
                "[Client:{}] Closing in progress code: {code}",
                client.session_id()
            )),
            _ => ctx.logger.info("WebSocketServer is closing..."),
        }
    });

    // ---- minecraft events --------------------------------------------------
    on!(mcwss, ctx, BlockBroken, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "block.broken",
            b.player.name,
            b.player.position,
            item_name(&b.block.namespace, &b.block.id),
            item_name(&b.tool.namespace, &b.tool.id)
        ));
    });

    on!(mcwss, ctx, BlockPlaced, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "block.placed",
            b.player.name,
            b.player.position,
            item_name(&b.block.namespace, &b.block.id),
            item_name(&b.tool.namespace, &b.tool.id)
        ));
    });

    on!(mcwss, ctx, EndOfDay, |d| {
        let b = &d.body;
        ctx.logger
            .info(&tr!(ctx, "end.of.day", b.player.name, b.player.position));
    });

    on!(mcwss, ctx, EntitySpawned, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "entity.spawned",
            b.player.name,
            b.player.position,
            b.mob.kind
        ));
    });

    on!(mcwss, ctx, ItemAcquired, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "item.acquired",
            b.player.name,
            b.player.position,
            b.count,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, ItemCrafted, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "item.crafted",
            b.player.name,
            b.player.position,
            b.count,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, ItemDestroyed, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "item.destroyed",
            b.player.name,
            b.player.position,
            b.count,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, ItemSmelted, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "item.smelted",
            b.player.name,
            b.player.position,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, ItemUsed, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "item.used",
            b.player.name,
            b.player.position,
            b.count,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, JukeboxUsed, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "jukebox.used",
            b.player.name,
            b.player.position,
            b.count,
            item_name(&b.item.namespace, &b.item.id)
        ));
    });

    on!(mcwss, ctx, MobInteracted, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "mob.interacted",
            b.player.name,
            b.player.position,
            format!("{} {}", b.mob.kind, b.mob.variant),
            b.interaction_type
        ));
    });

    on!(mcwss, ctx, MobKilled, |d| {
        let b = &d.body;
        let victim_name = if b.victim.name.is_empty() {
            "mob.killed.unnamed".to_string()
        } else {
            b.victim.name.clone()
        };
        ctx.logger.info(&tr!(
            ctx,
            "mob.killed",
            b.player.name,
            b.player.position,
            b.victim.position,
            b.victim.id,
            victim_name
        ));
    });

    on!(mcwss, ctx, PlayerBounced, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "player.bounced",
            b.player.name,
            b.player.position,
            item_name(&b.block.namespace, &b.block.id)
        ));
    });

    on!(mcwss, ctx, PlayerDied, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "player.died",
            b.player.name,
            b.player.position,
            format!("{} {}", b.killer.kind, b.killer.variant)
        ));
    });

    on!(mcwss, ctx, PlayerMessage, |d| {
        let b = &d.body;
        let receiver = if b.receiver.is_empty() {
            "player.message.all"
        } else {
            b.receiver.as_str()
        };
        ctx.logger.info(&tr!(
            ctx,
            "player.message",
            b.sender,
            receiver,
            b.kind,
            b.message
        ));
    });

    on!(mcwss, ctx, PlayerTeleported, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "player.teleported",
            b.player.name,
            b.player.position,
            b.meters_travelled
        ));
    });

    // `PlayerTransform` fires together with `PlayerTravelled`, so only the latter is logged.
    on!(mcwss, ctx, PlayerTravelled, |d| {
        let b = &d.body;
        ctx.logger.info(&tr!(
            ctx,
            "player.travelled",
            b.player.name,
            b.player.position,
            b.meters_travelled
        ));
    });

    on!(mcwss, ctx, UnknownMinecraftEvent, |d| {
        ctx.logger.warn(&d.raw.to_string());
    });

    on!(mcwss, ctx, CommandResponse, |d| {
        let Some(message) = d.body.status_message.as_deref().filter(|m| !m.is_empty()) else {
            return;
        };
        let _ = d.client.tell(tr!(ctx, "command.response", message));
    });

    // ---- built-in chat commands (`*/help`, `./cmd`, ...) --------------------
    on!(mcwss, ctx, PlayerMessage, |d| {
        let body = &d.body;
        if !body.receiver.is_empty() {
            return;
        }
        let client = &d.client;
        match body.message.as_str() {
            "*/help" => {
                let _ = client.tell(tr!(ctx, "command.help"));
            }
            "*/helph" => {
                let _ = client.tell(tr!(ctx, "command.helpf"));
            }
            "*/connect" => {
                let start = client.session_date();
                let elapsed = (chrono::Utc::now() - start).num_milliseconds() as f64 / 1000.0;
                let _ = client.tell(tr!(
                    ctx,
                    "command.connect",
                    elapsed,
                    start.timestamp_millis()
                ));
            }
            "*/about" => {
                let author = env!("CARGO_PKG_AUTHORS").split(' ').next().unwrap_or("");
                let ip = client
                    .remote_addr()
                    .map(|a| a.ip().to_string())
                    .unwrap_or_default();
                let _ = client.tell(tr!(
                    ctx,
                    "command.about",
                    author,
                    env!("CARGO_PKG_VERSION"),
                    ip
                ));
            }
            "*/clears" => {
                for _ in 0..100 {
                    let _ = client.tell("");
                }
            }
            message => {
                if let Some(cmd) = message.strip_prefix("./") {
                    let _ = client.run(cmd);
                } else if let Some(path) = message.strip_prefix("*/func ") {
                    let file = std::env::current_dir()
                        .unwrap_or_default()
                        .join(path.trim());
                    if !matches!(client.func(&file), Ok(true)) {
                        let _ = client.chatf(
                            tr!(ctx, "command.function", file.display()),
                            Some(TextColor::Red),
                            None,
                            CommandTarget::SelfPlayer,
                        );
                    }
                }
            }
        }
    });

    // ---- stdin: `restart` / `stop` -------------------------------------------
    {
        let mcwss = mcwss.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(tokio::io::stdin()).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                match line.trim() {
                    "restart" => {
                        mcwss.stop().await;
                        if let Err(e) = mcwss.start().await {
                            ctx.logger.error(&format!("[Server] {e}"));
                        }
                    }
                    "stop" => {
                        mcwss.stop().await;
                        std::process::exit(0);
                    }
                    _ => {}
                }
            }
        });
    }

    ctx.logger.debug("starting server");
    mcwss.start().await?;

    tokio::signal::ctrl_c().await?;
    mcwss.stop().await;
    Ok(())
}
