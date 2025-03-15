mod packet;

use anyhow::Context;
use clap::Parser;
use mccli::fetch_server_info;
use mccli::types;
use std::net::ToSocketAddrs;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[derive(Parser)]
struct Args {
    addr: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    let Args { addr } = Args::parse();

    let info = fetch_server_info(
        addr.to_socket_addrs()
            .or_else(|_| format!("{addr}:25565").to_socket_addrs())
            .context("getting socket address")?
            .next()
            .unwrap(),
    )
    .await?;
    println!("Server is online:");
    println!("Version: {}", info.version.name);
    println!("Players: {}/{}", info.players.online, info.players.max);
    for p in info.players.sample {
        println!("  - {}", p.name);
    }
    if let Some(modinfo) = info.modinfo {
        println!("mod type: {}", modinfo.r#type);
        if !modinfo.mod_list.is_empty() {
            println!("Mod list:");
            for m in modinfo.mod_list {
                println!("  - {}", m)
            }
        }
    }
    println!("Description:");
    match info.description {
        types::server::Description::Text(t) => println!("{t}"),
        types::server::Description::Colored(colored_text) => {
            fn print_color_text(c: types::server::ColoredText) {
                print!("{}", c.text);
                for e in c.extra {
                    print_color_text(e)
                }
            }
            print_color_text(colored_text);
            println!();
        }
    }
    Ok(())
}
