//! Minimal server built on the `mcwss` library.
//!
//! Run with `cargo run --example basic`, then in Minecraft: `/wsserver ws://localhost:2333`

use mcwss::{Event, EventKind, Mcwss, McwssConfig};

#[tokio::main]
async fn main() -> mcwss::Result<()> {
    let mcwss = Mcwss::new(McwssConfig::new(2333));

    mcwss.on(EventKind::Ready, |event| {
        if let Event::Ready(data) = event {
            println!(
                "WebSocketServer started at ws://localhost:{}",
                data.addr.port()
            );
        }
    });

    mcwss.on(EventKind::Dispose, |_| println!("WebSocketServer stopped"));

    mcwss.on(EventKind::Error, |event| {
        if let Event::Error(data) = event {
            let who = data.client.as_ref().map_or("[Server]".to_string(), |c| {
                format!("[Client:{}]", c.session_id())
            });
            eprintln!("{who} {} {}", data.name, data.message);
        }
    });

    mcwss.on(EventKind::Connection, |event| {
        if let Event::Connection(data) = event {
            println!(
                "[Client:{}] new connection from {:?}",
                data.client.session_id(),
                data.client.remote_addr()
            );
            let _ = data.client.chat("Hello from Rust!");
        }
    });

    mcwss.on(EventKind::Close, |event| {
        if let Event::Close(data) = event {
            match (&data.client, data.code) {
                (Some(c), Some(code)) => {
                    println!(
                        "[Client:{}] Closing in progress code: {code}",
                        c.session_id()
                    )
                }
                _ => println!("WebSocketServer is closing..."),
            }
        }
    });

    // Listening to a Minecraft event automatically subscribes every client to it.
    mcwss.on(EventKind::BlockBroken, |event| {
        if let Event::BlockBroken(data) = event {
            let body = &data.body;
            println!(
                "{} broke {}:{} at {}",
                body.player.name, body.block.namespace, body.block.id, body.player.position
            );
        }
    });

    mcwss.on(EventKind::EndOfDay, |event| {
        if let Event::EndOfDay(data) = event {
            println!("{} just experienced nightfall", data.body.player.name);
        }
    });

    mcwss.start().await?;
    tokio::signal::ctrl_c().await?;
    mcwss.stop().await;
    Ok(())
}
