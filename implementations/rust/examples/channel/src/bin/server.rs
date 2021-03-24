use channel_examples::server_worker::Server;
use ockam::Result;
use ockam_channel::channel_listener::{XXChannelListener, XX_CHANNEL_LISTENER_ADDRESS};
use ockam_transport_tcp::TcpRouter;
use std::net::SocketAddr;
use std::str::FromStr;
use tracing::info;

#[ockam::node]
async fn main(ctx: ockam::Context) -> Result<()> {
    let xx_channel_listener = XXChannelListener::new(Vec::new().into());
    ctx.start_worker(XX_CHANNEL_LISTENER_ADDRESS, xx_channel_listener)
        .await
        .unwrap();

    let server = Server {};

    // Create the responder worker
    ctx.start_worker("echo_server", server).await?;

    // Get either the default socket address, or a user-input
    let bind_addr = SocketAddr::from_str("127.0.0.1:4050").unwrap();
    info!("Binding to: {}", bind_addr);

    // Create a new _binding_ TcpRouter
    let _r = TcpRouter::bind(&ctx, bind_addr).await?;

    // Crashes: ctx.stop().await

    Ok(())
}
