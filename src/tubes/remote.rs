use crate::tubes::sock::Sock;
use crate::tubes::tube::Tube;
use crate::{info, Tubeable};
use std::io;
use std::net::TcpStream;
use std::time::Duration;

/// A generic TCP client struct
///
/// # Examples
/// ```
/// use pwn::tubes::remote::Remote;
/// use pwn::tubes::tube::Tube;
/// let mut sock = Remote::new("tcpbin.com", 4242).unwrap();
/// let data = b"test";
/// sock.sendline(*data);
/// ```
#[derive(Clone)]
pub struct Remote {
    sock: Sock,
    _host: String,
    _port: i32,
}

impl Remote {
    /// Create a TCP client connection.
    pub fn new<T: ToString, T2: Into<i32>>(host: T, port: T2) -> io::Result<Remote> {
        let port = port.into();
        let conn_str = format!("{}:{}", host.to_string(), port);
        info!("Opening connection to {}", conn_str);
        Ok(Remote {
            sock: Sock::new(TcpStream::connect(conn_str)?),
            _host: host.to_string(),
            _port: port,
        })
    }
}

impl Tubeable for Remote {
    fn get_receiver(&self) -> std::sync::mpsc::Receiver<Vec<u8>> {
        todo!()
    }

    fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        todo!()
    }
}
