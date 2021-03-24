use crate::channel::{Channel, ChannelMessage};
use async_trait::async_trait;
use ockam::{Address, Context, Worker};
use ockam_core::{Result, Routed};
use serde::{Deserialize, Serialize};

pub const XX_CHANNEL_LISTENER_ADDRESS: &str = "xx_channel_listener";

pub struct XXChannelListener {
    key_exchange_factory_address: Address,
}

impl XXChannelListener {
    pub fn new(key_exchange_factory_address: Address) -> Self {
        Self {
            key_exchange_factory_address,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ChannelListenerMessage {
    CreateResponderChannel {
        channel_id: String,
        payload: Vec<u8>,
    },
}

#[async_trait]
impl Worker for XXChannelListener {
    type Message = ChannelListenerMessage;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let reply = msg.reply().clone();
        match msg.take() {
            ChannelListenerMessage::CreateResponderChannel {
                channel_id,
                payload,
            } => {
                let address = Channel::get_address(channel_id.clone());

                let channel = Channel::new(
                    self.key_exchange_factory_address.clone(),
                    false,
                    reply,
                    channel_id,
                    None,
                );

                ctx.start_worker(address.clone(), channel).await?;

                ctx.send_message(address, ChannelMessage::KeyExchange { payload })
                    .await
            }
        }
    }
}
