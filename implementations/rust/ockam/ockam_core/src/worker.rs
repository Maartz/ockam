use crate::{lib::Box, Result, TransportMessage};
use async_trait::async_trait;

/// Base ockam worker trait.
#[async_trait]
pub trait Worker: Send + 'static {
    /// The API and other resources available for the worker during message processing.
    type Context: Send + 'static;

    /// Override initialisation behaviour
    async fn initialize(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    /// Override shutdown behaviour
    fn shutdown(&mut self, _context: &mut Self::Context) -> Result<()> {
        Ok(())
    }

    /// Try to open and handle a typed message
    async fn handle_message(
        &mut self,
        _context: &mut Self::Context,
        _msg: TransportMessage
    ) -> Result<()> {
        Ok(())
    }
}
