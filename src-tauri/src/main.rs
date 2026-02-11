// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(name = "chatr", about = "P2P Decentralized Chat")]
struct Cli {
    /// Run without GUI (API server only)
    #[arg(long)]
    headless: bool,

    /// API server port
    #[arg(long, default_value = "9847")]
    port: u16,

    /// Custom data directory
    #[arg(long)]
    data_dir: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    if cli.headless {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(chatr_lib::run_headless(
            cli.data_dir.as_deref(),
            cli.port,
        ));
    } else {
        chatr_lib::run_with_opts(cli.data_dir.as_deref(), cli.port);
    }
}
