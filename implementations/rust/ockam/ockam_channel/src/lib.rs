use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum KeyExchangeRequestMessage {
    Payload(Vec<u8>),
    InitiatorFirstMessage,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Keys {
    h: [u8; 32],
    encrypt_key: usize,
    decrypt_key: usize,
}

impl Keys {
    pub fn new(h: [u8; 32], encrypt_key: usize, decrypt_key: usize) -> Self {
        Keys {
            h,
            encrypt_key,
            decrypt_key,
        }
    }
}

impl Keys {
    pub fn h(&self) -> [u8; 32] {
        self.h
    }
    pub fn encrypt_key(&self) -> usize {
        self.encrypt_key
    }
    pub fn decrypt_key(&self) -> usize {
        self.decrypt_key
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct KeyExchangeResponseMessage {
    payload: Option<Vec<u8>>,
    keys: Option<Keys>,
}

impl KeyExchangeResponseMessage {
    pub fn new(payload: Option<Vec<u8>>, keys: Option<Keys>) -> Self {
        KeyExchangeResponseMessage { payload, keys }
    }
}

pub mod channel;
pub mod channel_listener;
pub mod initiator;
pub mod responder;

mod error;
pub use error::*;
