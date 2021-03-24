use crate::KeyExchangeRequestMessage;
use crate::{ChannelError, KeyExchangeResponseMessage, Keys};
use async_trait::async_trait;
use ockam::{Context, Worker};
use ockam_core::{Result, Routed};
use ockam_key_exchange_core::KeyExchanger;
use ockam_key_exchange_xx::Initiator;

pub struct XInitiator {
    initiator: Option<Initiator>,
}

impl XInitiator {
    pub fn new(initiator: Initiator) -> Self {
        XInitiator {
            initiator: Some(initiator),
        }
    }
}

#[async_trait]
impl Worker for XInitiator {
    type Message = KeyExchangeRequestMessage;
    type Context = Context;

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let reply = msg.reply();
        match msg.take() {
            KeyExchangeRequestMessage::InitiatorFirstMessage => {
                let initiator;
                if let Some(i) = self.initiator.as_mut() {
                    initiator = i;
                } else {
                    return Err(ChannelError::InvalidInternalState.into());
                }

                // discard any payload and get the next message
                let response = KeyExchangeResponseMessage::new(Some(initiator.process(&[])?), None);

                ctx.send_message(reply.clone(), response).await?;
            }
            KeyExchangeRequestMessage::Payload(payload) => {
                let initiator;
                if let Some(i) = self.initiator.as_mut() {
                    initiator = i;
                } else {
                    return Err(ChannelError::InvalidInternalState.into());
                }

                // discard any payload and get the next message
                let _ = initiator.process(&payload)?;

                let new_payload = if !initiator.is_complete() {
                    Some(initiator.process(&[])?)
                } else {
                    None
                };

                let mut should_stop = false;

                let keys = if initiator.is_complete() {
                    let initiator;
                    if let Some(i) = self.initiator.take() {
                        initiator = i;
                    } else {
                        return Err(ChannelError::InvalidInternalState.into());
                    }

                    let keys = initiator.finalize()?;
                    let keys = Keys::new(
                        keys.h().clone(),
                        keys.encrypt_key().index(),
                        keys.decrypt_key().index(),
                    );
                    should_stop = true;

                    Some(keys)
                } else {
                    None
                };

                ctx.send_message(reply, KeyExchangeResponseMessage::new(new_payload, keys))
                    .await?;

                if should_stop {
                    ctx.stop_worker(ctx.address()).await?;
                }
            }
        }
        Ok(())
    }
}
