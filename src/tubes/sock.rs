use crate::tubes::{buffer::Buffer, tube::Tube};
use crate::Tubeable;
use std::io;
use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::time::Duration;

/// A generic TCP socket that can be a client or server.
pub struct Sock {
    sock: TcpStream,
    buffer: Buffer,
}

impl Sock {
    /// Create a `Sock` from a `TcpStream` with an internal [`Buffer`].
    pub fn new(sock: TcpStream) -> Self {
        Self {
            sock,
            buffer: Buffer::new(),
        }
    }
}

impl Tubeable for Sock {
    fn get_receiver(&self) -> std::sync::mpsc::Receiver<std::collections::VecDeque<u8>> {
        todo!()
    }

    fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        todo!()
    }
}

impl Clone for Sock {
    fn clone(&self) -> Self {
        Sock {
            sock: self.sock.try_clone().unwrap(),
            buffer: self.buffer.clone(),
        }
    }
}
