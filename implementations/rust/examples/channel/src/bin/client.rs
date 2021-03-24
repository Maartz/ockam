use channel_examples::client_worker::Client;
use ockam::{Result, Route};
use ockam_channel::channel::{Channel, KeyExchangeCompleted};
use ockam_channel::channel_listener::XX_CHANNEL_LISTENER_ADDRESS;
use ockam_transport_tcp::{start_tcp_worker, TcpRouter};
use std::net::SocketAddr;
use std::str::FromStr;
use tokio::time::{sleep, Duration};

#[ockam::node]
async fn main(mut ctx: ockam::Context) -> Result<()> {
    // Create and register a TcpRouter
    let rh = TcpRouter::register(&ctx).await?;

    let responder_addr = SocketAddr::from_str("127.0.0.1:4050").unwrap();
    // Create and register a connection worker pair
    let w_pair = start_tcp_worker(&ctx, responder_addr).await?;
    rh.register(&w_pair).await?;

    let channel_id = "random_id".to_string();

    Channel::start_initiator_channel(
        &ctx,
        channel_id.clone(),
        Route::new()
            .append(format!("1#{}", responder_addr))
            .append(XX_CHANNEL_LISTENER_ADDRESS.to_string()),
    )
    .await?;

    // FIXME: Either of next 2 lines messes things up on server side
    // let _ = ctx.receive_match(|m: &KeyExchangeCompleted| m.channel_id() == channel_id).await?;
    // let _ = ctx.receive::<KeyExchangeCompleted>().await?;
    sleep(Duration::new(2, 0)).await;

    let client = Client::new(channel_id);

    ctx.start_worker("echo_client", client).await?;

    // Crashes: ctx.stop().await

    Ok(())
}
