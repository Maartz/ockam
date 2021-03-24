use ockam::{async_worker, Context, Result, Route, Routed, Worker};
use ockam_channel::channel::{Channel, ChannelMessage};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::time::Duration;
use tracing::info;

pub struct Client {
    channel_id: String,
}

impl Client {
    pub fn new(channel_id: String) -> Self {
        Client { channel_id }
    }
}

#[async_worker]
impl Worker for Client {
    type Context = Context;
    type Message = String;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        ctx.send_message(ctx.address(), "recursion".to_string())
            .await
    }

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<String>) -> Result<()> {
        let str = msg.take();
        if str == "recursion" {
            let rand_string: String = thread_rng()
                .sample_iter(&Alphanumeric)
                .take(30)
                .map(char::from)
                .collect();
            info!("Client sent message: {}", rand_string);
            ctx.send_message(ctx.address(), "recursion".to_string())
                .await?;
            ctx.send_message(
                Route::new()
                    .append(Channel::get_address(self.channel_id.clone()))
                    .append("echo_server"),
                ChannelMessage::encrypt(rand_string)?,
            )
            .await?;
            tokio::time::sleep(Duration::from_secs(2)).await;
        } else {
            info!("Client received msg: {}", str);
        }
        Ok(())
    }
}
