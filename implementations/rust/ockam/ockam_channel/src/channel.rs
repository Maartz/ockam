use crate::channel_listener::ChannelListenerMessage;
use crate::initiator::XInitiator;
use crate::responder::XResponder;
use crate::{ChannelError, KeyExchangeRequestMessage, KeyExchangeResponseMessage};
use async_trait::async_trait;
use ockam::{Address, Context, Message, TransportMessage, Worker};
use ockam_core::{Result, Route, Routed};
use ockam_key_exchange_core::NewKeyExchanger;
use ockam_key_exchange_xx::XXNewKeyExchanger;
use ockam_vault::SoftwareVault;
use ockam_vault_core::{Secret, SymmetricVault};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tracing::debug;

struct ChannelKeys {
    _h: [u8; 32],
    encrypt_key: Secret,
    decrypt_key: Secret,
    nonce: u16,
}

pub struct Channel {
    _key_exchange_factory_address: Address,
    is_initiator: bool,
    sent_first_msg: bool,
    received_first_msg: bool,
    route: Route,
    channel_id: String,
    key_exchange_route: Option<Route>, // this address is used to send messages to key exchange worker
    keys: Option<ChannelKeys>,
    key_exchange_completed_callback_route: Option<Route>,
    vault: Arc<Mutex<SoftwareVault>>,
}

impl Channel {
    pub fn new(
        _key_exchange_factory_address: Address,
        is_initiator: bool,
        route: Route,
        channel_id: String,
        key_exchange_completed_callback_route: Option<Route>,
    ) -> Self {
        // TODO: Replace with worker
        let vault = Arc::new(Mutex::new(SoftwareVault::new()));
        Channel {
            _key_exchange_factory_address,
            is_initiator,
            sent_first_msg: false,
            received_first_msg: false,
            route,
            channel_id,
            key_exchange_route: None,
            keys: None,
            key_exchange_completed_callback_route,
            vault,
        }
    }

    pub fn get_address(channel_id: String) -> Address {
        format!("channel/{}", channel_id).as_bytes().to_vec().into()
    }

    pub async fn start_initiator_channel<A: Into<Route>>(
        ctx: &Context,
        channel_id: String,
        route: A,
    ) -> Result<()> {
        let address = Self::get_address(channel_id.clone());

        let channel = Channel::new(
            Vec::new().into(),
            true,
            route.into(),
            channel_id,
            Some(Route::new().append(ctx.address()).into()),
        );

        ctx.start_worker(address, channel).await?;

        Ok(())
    }

    fn convert_nonce_u16(nonce: u16) -> ([u8; 2], [u8; 12]) {
        let mut n: [u8; 12] = [0; 12];
        let b: [u8; 2] = nonce.to_be_bytes();
        n[10] = b[0];
        n[11] = b[1];

        (b, n)
    }

    fn convert_nonce_small(b: &[u8]) -> Result<[u8; 12]> {
        if b.len() != 2 {
            return Err(ChannelError::InvalidNonce.into());
        }
        let mut n: [u8; 12] = [0; 12];
        n[10] = b[0];
        n[11] = b[1];

        Ok(n)
    }

    fn get_keys(keys: &mut Option<ChannelKeys>) -> Result<&mut ChannelKeys> {
        if let Some(k) = keys.as_mut() {
            Ok(k)
        } else {
            // TODO: Requeue
            return Err(ChannelError::KeyExchangeNotComplete.into());
        }
    }

    async fn handle_key_exchange_local(
        &mut self,
        ctx: &mut <Self as Worker>::Context,
        response: KeyExchangeResponseMessage,
    ) -> Result<()> {
        debug!("Channel received KeyExchangeLocal");

        if let Some(payload) = response.payload {
            debug!("Channel received Payload");
            if self.is_initiator && !self.sent_first_msg {
                ctx.send_message(
                    self.route.clone(),
                    ChannelListenerMessage::CreateResponderChannel {
                        channel_id: self.channel_id.clone(),
                        payload,
                    },
                )
                .await?;
            } else {
                ctx.send_message(self.route.clone(), ChannelMessage::KeyExchange { payload })
                    .await?;
            };

            self.sent_first_msg = true;
        }
        if let Some(keys) = response.keys {
            debug!("Channel received ExchangeComplete");

            if self.keys.is_some() {
                return Err(ChannelError::InvalidInternalState.into());
            }

            self.keys = Some(ChannelKeys {
                _h: keys.h,
                encrypt_key: Secret::new(keys.encrypt_key),
                decrypt_key: Secret::new(keys.decrypt_key),
                nonce: 1,
            });

            if let Some(r) = self.key_exchange_completed_callback_route.take() {
                ctx.send_message(
                    r,
                    KeyExchangeCompleted {
                        channel_id: self.channel_id.clone(),
                    },
                )
                .await?;
            }
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum ChannelMessage {
    KeyExchange { payload: Vec<u8> },
    Encrypt { m: Vec<u8> },
    Decrypt { payload: Vec<u8> },
}

impl ChannelMessage {
    pub fn encrypt<M: Message>(m: M) -> Result<ChannelMessage> {
        let m = ChannelMessage::Encrypt { m: m.encode()? };

        Ok(m)
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyExchangeCompleted {
    channel_id: String,
}

impl KeyExchangeCompleted {
    pub fn channel_id(&self) -> &str {
        &self.channel_id
    }
}

#[async_trait]
impl Worker for Channel {
    type Message = ChannelMessage;
    type Context = Context;

    async fn initialize(&mut self, ctx: &mut Self::Context) -> Result<()> {
        // TODO: Replace key_exchanger and vault with worker
        let key_exchanger = XXNewKeyExchanger::new(self.vault.clone(), self.vault.clone());

        let key_exchange_addr: Address = format!("channel/{}/kex", self.channel_id)
            .as_bytes()
            .to_vec()
            .into();

        let key_exchange_route: Route = Route::new().append(key_exchange_addr.clone()).into();
        self.key_exchange_route = Some(key_exchange_route.clone());

        if self.is_initiator {
            let initiator = key_exchanger.initiator();
            let initiator = XInitiator::new(initiator);

            ctx.start_worker(key_exchange_addr, initiator).await?;

            ctx.send_message(
                key_exchange_route,
                KeyExchangeRequestMessage::InitiatorFirstMessage,
            )
            .await?;

            let m = ctx.receive::<KeyExchangeResponseMessage>().await?.take();

            self.handle_key_exchange_local(ctx, m).await?;
        } else {
            let responder = key_exchanger.responder();
            let responder = XResponder::new(responder);

            ctx.start_worker(key_exchange_addr, responder).await?;
        }

        Ok(())
    }

    async fn handle_message(
        &mut self,
        ctx: &mut Self::Context,
        msg: Routed<Self::Message>,
    ) -> Result<()> {
        let reply = msg.reply().clone();
        let onward_route = msg.onward();
        match msg.take() {
            ChannelMessage::KeyExchange { payload } => {
                debug!("Channel received KeyExchangeRemote");
                let key_exchange_route;
                if let Some(a) = self.key_exchange_route.clone() {
                    key_exchange_route = a;
                } else {
                    return Err(ChannelError::InvalidInternalState.into());
                }
                if self.is_initiator || self.received_first_msg {
                    self.route = reply;
                }
                self.received_first_msg = true;

                ctx.send_message(
                    key_exchange_route,
                    KeyExchangeRequestMessage::Payload(payload),
                )
                .await?;

                let m = ctx.receive::<KeyExchangeResponseMessage>().await?.take();

                self.handle_key_exchange_local(ctx, m).await?;
            }
            ChannelMessage::Encrypt { m } => {
                debug!("Channel received Encrypt");

                let onward_addresses = onward_route.inner();
                let mut onward_route = Route::new();
                for address in &onward_addresses[1..] {
                    onward_route = onward_route.append(address.clone());
                }

                let msg = TransportMessage {
                    version: 1,
                    onward: onward_route.into(),
                    return_: reply,
                    payload: m,
                };
                let payload = msg.encode()?;

                let payload = {
                    let keys = Self::get_keys(&mut self.keys)?;

                    let nonce = keys.nonce;

                    if nonce == u16::max_value() {
                        return Err(ChannelError::InvalidNonce.into());
                    }

                    keys.nonce += 1;

                    let (small_nonce, nonce) = Self::convert_nonce_u16(nonce);

                    let mut vault = self.vault.lock().unwrap();
                    let mut cipher_text = vault.aead_aes_gcm_encrypt(
                        &keys.encrypt_key,
                        payload.as_slice(),
                        &nonce,
                        &[],
                    )?;

                    let mut res = Vec::new();
                    res.extend_from_slice(&small_nonce);
                    res.append(&mut cipher_text);

                    res
                };

                ctx.send_message(self.route.clone(), ChannelMessage::Decrypt { payload })
                    .await?;
            }
            ChannelMessage::Decrypt { payload } => {
                debug!("Channel received RouterRemote");
                let payload = {
                    let keys = Self::get_keys(&mut self.keys)?;

                    if payload.len() < 2 {
                        return Err(ChannelError::InvalidNonce.into());
                    }

                    let nonce = Self::convert_nonce_small(&payload.as_slice()[..2])?;

                    let mut vault = self.vault.lock().unwrap();
                    let plain_text = vault.aead_aes_gcm_decrypt(
                        &keys.decrypt_key,
                        &payload[2..],
                        &nonce,
                        &[],
                    )?;

                    plain_text
                };

                let mut transport_message = TransportMessage::decode(&payload)?;

                transport_message
                    .return_
                    .modify()
                    .prepend(Channel::get_address(self.channel_id.clone()));

                ctx.forward_message(transport_message).await?;
            }
        }
        Ok(())
    }
}
