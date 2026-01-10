#![cfg(feature = "3ds")]

// re-implementation of tokio-tungstenite
// pub use tungstenite::protocol::frame::CloseFrame as CloseFrame;
// so we dont have to clean up lifetimes.
pub type CloseFrame<'a> = tungstenite::protocol::frame::CloseFrame;

pub use tungstenite::Message;

// https://docs.rs/tungstenite/latest/tungstenite/error/enum.Error.html
pub use tungstenite::error::Error as TungsteniteError;
pub use tungstenite::protocol::WebSocketConfig as WebSocketConfig;
pub use tungstenite::stream::MaybeTlsStream;
pub use tungstenite::client::connect_with_config;
use tungstenite::WebSocket;


use std::pin::Pin;
use std::task::{Context, Poll};
use futures::stream::Stream;

// need a stream for the websocket otherwise .next() no workie
pub struct WebSocketStream<S>(pub WebSocket<S>);
impl<S> Stream for WebSocketStream<S>
where
    S: std::io::Read + std::io::Write + Unpin,
{
    type Item = Result<Message, TungsteniteError>;

    fn poll_next(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    match self.get_mut().0.read() {
        Ok(msg) => Poll::Ready(Some(Ok(msg))),
        Err(tungstenite::Error::ConnectionClosed) => Poll::Ready(None),
        Err(e) => Poll::Ready(Some(Err(e))),
    }
}
}
impl<S> futures::Sink<Message> for WebSocketStream<S>
where
    S: Read + Write + Unpin,
{
    type Error = tungstenite::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.0.send(item)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

use std::io::{Read, Write};

impl<S> WebSocketStream<S> 
where 
    S: Read + Write + Unpin 
{
    pub fn send(&mut self, msg: Message) -> Result<(), tungstenite::Error> {
        self.0.send(msg)
    }

    pub fn read(&mut self) -> Result<Message, tungstenite::Error> {
        self.0.read()
    }
}