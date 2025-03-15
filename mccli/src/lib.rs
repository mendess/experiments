mod packet;

use packet::Packet;
use std::net::SocketAddr;
use tokio::net::TcpStream;

pub use packet::types;

pub async fn fetch_server_info(addr: SocketAddr) -> anyhow::Result<types::server::Status> {
    tracing::info!("connecting to: {addr}");
    let mut socket = TcpStream::connect(addr).await?;

    tracing::info!("sending handshake");
    Packet::handshake(769).await.write(&mut socket).await?;

    tracing::info!("requesting status");
    Packet::status_request().write(&mut socket).await?;

    tracing::info!("reading status");
    let response = Packet::read(&mut socket).await?;

    let text = response.reader().next::<types::String>().await?;

    tracing::debug!(%text, "text");

    Ok(serde_json::from_str(&text)?)
}
