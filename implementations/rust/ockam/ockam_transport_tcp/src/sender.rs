use crate::TcpError;
use ockam::{async_worker, Context, Result, Routed, TransportMessage, Worker};
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};

/// A TCP sending message worker
///
/// Create this worker type by calling
/// [`start_tcp_worker`](crate::start_tcp_worker)!
///
/// This half of the worker is created when spawning a new connection
/// worker pair, and listens for messages from the node message system
/// to dispatch to a remote peer.
pub struct TcpSendWorker {
    pub(crate) tx: OwnedWriteHalf,
}

fn prepare_message(msg: TransportMessage) -> Result<Vec<u8>> {
    let mut msg_buf = serde_bare::to_vec(&msg).map_err(|_| TcpError::SendBadMessage)?;

    // Create a buffer that includes the message length in big endian
    let mut len = (msg_buf.len() as u16).to_be_bytes().to_vec();

    // Fun fact: reversing a vector in place, appending the length,
    // and then reversing it again is faster for large message sizes
    // than adding the large chunk of data.
    //
    // https://play.rust-lang.org/?version=stable&mode=release&edition=2018&gist=8669a640004ac85c7be38b19e3e73dcb
    msg_buf.reverse();
    len.reverse();
    msg_buf.append(&mut len);
    msg_buf.reverse();

    Ok(msg_buf)
}

#[async_worker]
impl Worker for TcpSendWorker {
    type Context = Context;
    type Message = TransportMessage;

    // TcpSendWorker will receive messages from the TcpRouter to send
    // across the TcpStream to our friend
    async fn handle_message(
        &mut self,
        _: &mut Context,
        msg: Routed<TransportMessage>,
    ) -> Result<()> {
        // Create a message buffer with pre-pended length
        let msg = prepare_message(msg.take())?;

        match self.tx.write(msg.as_slice()).await {
            Ok(_) => Ok(()),
            // TODO: match different error types here!
            Err(_) => Err(TcpError::SendBadMessage.into()),
        }
    }
}
