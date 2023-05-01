use crate::Tubeable;
use std::io;
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

/// A generic TCP socket that can be a client or server.
pub struct Sock {
    sock: TcpStream,
}

impl Sock {
    /// Create a `Sock` from a `TcpStream` with an internal [`Buffer`].
    pub fn new(sock: TcpStream) -> Self {
        Self { sock }
    }
}

impl Tubeable for Sock {
    fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        todo!()
    }

    fn get_receiver(&self) -> Receiver<Vec<u8>> {
        todo!()
    }
}

impl Clone for Sock {
    fn clone(&self) -> Self {
        Sock {
            sock: self.sock.try_clone().unwrap(),
        }
    }
}
