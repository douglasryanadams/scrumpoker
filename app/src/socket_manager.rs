use tungstenite::{WebSocket, Message};
use std::net::TcpStream;
use log::{warn};

pub(crate) struct SocketManager<'a> {
    socket: &'a WebSocket<TcpStream>
}

impl SocketManager<'_> {
    pub(crate) fn new(web_socket: & WebSocket<TcpStream>) -> SocketManager {
        return SocketManager { socket: web_socket };
    }

    pub(crate) fn read_message(&self) -> Option<Message> {
        match self.socket.read_message() {
            Ok(message) => return Some(message),
            Err(error) => match error {
                tungstenite::error::Error::AlreadyClosed => {
                    // Nothing to be done
                    None
                }
                _ => {
                    warn!("  ->> error reading message: {:?}", error);
                    None
                }
            }
        }
    }

    pub(crate) fn send_message(&self, message_string: String) {
        let message = Message::Text(message_string);
        self.socket.write_message(message);
    }
}