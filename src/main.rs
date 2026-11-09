mod i18n;
mod logger;
mod program;

use clap::Parser;

use crate::{
    i18n::{DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES},
    logger::Level,
};

const DEFAULT_PORT: u16 = 1;

/// Minecraft Bedrock Websocket Server Framework
#[derive(Parser)]
#[command(name = "mcwss", disable_version_flag = true)]
struct Cli {
    /// Set websocket server port
    #[arg(long, value_name = "num")]
    port: Option<u16>,

    /// Set logger level, debug or build
    #[arg(long, value_name = "type")]
    mode: Option<String>,

    /// Set view language: en_US, ja_JP, zh_TW, zh_CN
    #[arg(long, value_name = "locale")]
    lang: Option<String>,

    /// Display version number
    #[arg(short = 'v', long = "version")]
    version: bool,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("mcwss/{}", env!("CARGO_PKG_VERSION"));
        return;
    }

    let level = if cli.mode.as_deref() == Some("debug") {
        Level::Trace
    } else {
        Level::Info
    };
    let lang = match cli.lang.as_deref() {
        Some(l) if SUPPORTED_LANGUAGES.contains(&l) => l,
        _ => DEFAULT_LANGUAGE,
    };

    if let Err(e) = program::run(cli.port.unwrap_or(DEFAULT_PORT), level, lang).await {
        eprintln!("[Server] {e}");
        std::process::exit(1);
    }
}
