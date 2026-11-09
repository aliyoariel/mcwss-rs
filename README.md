# Mcwss (Rust)

A simple Minecraft Bedrock Edition websocket server framework, rewritten in Rust.
It is both a **library** (`mcwss` crate) and a **CLI** (`mcwss` binary).

The core `Mcwss` is an event emitter with many listenable events. Every event carries a
`Client` handle to interact with the game that sent it.

## Usage (CLI)

```
cargo run --release
cargo run --release -- --port 2333 --mode debug --lang zh_CN
```

Start Minecraft Bedrock and enter:

```
/wsserver ws://localhost:1
# or:
/connect ws://localhost:1
```

> The default port is `1` (same as the original). On Linux/macOS ports below 1024 need
> elevated privileges, so pass `--port 2333` (or similar) for normal use.

Options:

- `--port <num>` websocket server port
- `--mode <type>` `debug` or `build` (logger level)
- `--lang <locale>` `en_US`, `ja_JP`, `zh_TW`, `zh_CN`
- `-v, --version`, `-h, --help`

Type `restart` or `stop` in the terminal to restart/stop the server.

### Built-in chat commands

- `*/help` show command help
- `*/helph` show hidden command help
- `*/connect` show connection time
- `*/about` show about information
- `*/clears` clear chat
- `*/func <path>` run a `.mcfunction` file
- `./<command>` run a game command (e.g. `./closewebsocket`, `./agent`)

## Library

```rust
use mcwss::{Event, EventKind, Mcwss, McwssConfig};

#[tokio::main]
async fn main() -> mcwss::Result<()> {
    let mcwss = Mcwss::new(McwssConfig::new(2333));

    mcwss.on(EventKind::BlockBroken, |event| {
        if let Event::BlockBroken(data) = event {
            println!("{} broke {}", data.body.player.name, data.body.block.id);
            let _ = data.client.chat("nice!");
        }
    });

    mcwss.start().await?;
    tokio::signal::ctrl_c().await?;
    mcwss.stop().await;
    Ok(())
}
```

More in [`examples/basic.rs`](examples/basic.rs) and `src/program.rs`.

### Events

Lifecycle: `Ready`, `Dispose`, `Error`, `Connection`, `Message`, `Close`

Minecraft: `BlockBroken`, `BlockPlaced`, `EndOfDay`, `EntitySpawned`, `ItemAcquired`,
`ItemCrafted`, `ItemDestroyed`, `ItemSmelted`, `ItemUsed`, `JukeboxUsed`, `MobInteracted`,
`MobKilled`, `PlayerBounced`, `PlayerDied`, `PlayerMessage`, `PlayerTeleported`,
`PlayerTransform`, `PlayerTravelled`, `UnknownMinecraftEvent`, `CommandResponse`

Listening to a Minecraft event automatically subscribes every connecting client to it.

### API

`Mcwss`: `new(config)`, `start().await`, `stop().await`, `on(kind, f)`, `once(kind, f)`,
`off(id)`, `off_all(kind)`, `emit(event)`, `parallel(event).await`, `clients()`.

`Client`: `session_id()`, `session_date()`, `remote_addr()`, `send(&packet)`, `close()`,
`subscribe(name)`, `unsubscribe(name)`, `run(cmd)`, `run_parts(parts)`, `chat(msg)`,
`chatf(msg, color, sender, target)`, `tell(msg)`, `func(path)`.

## License

GNU General Public License v3.0, same as the original project.
