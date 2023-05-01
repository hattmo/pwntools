use crate::tubes::sock::Sock;
use crate::Tubeable;
use once_cell::sync::OnceCell;
use std::io;
use std::net::{SocketAddr, TcpListener};

/// A TCP listener which is connected to a [`Sock`]
///
/// # Example
/// Listen on all interfaces, on an OS-selected TCP port
/// ```
/// use pwn::tubes::listen::Listen;
/// let mut listener = Listen::new(Some("0.0.0.0"), None);
/// ```
pub struct Listen {
    /// The TCP listener we bound to
    listener: TcpListener,
    /// The TCP socket opened for communication
    sock: OnceCell<Sock>,
    /// The [`SocketAddr`] we're listening on
    pub addr: SocketAddr,
}

impl Listen {
    /// Create a TCP listener. By default, it will listen on all interfaces, and
    /// a port randomly chosen by the OS.
    pub fn new<T: ToString>(host: Option<T>, port: Option<i32>) -> io::Result<Self> {
        let host = match host {
            Some(h) => h.to_string(),
            None => "0.0.0.0".to_string(),
        };
        let port = match port {
            Some(p) => format!("{}", p),
            None => "0".to_string(),
        };

        let listener = TcpListener::bind(format!("{}:{}", host, port))?;
        let addr = listener.local_addr()?;
        Ok(Listen {
            listener,
            sock: OnceCell::new(),
            addr,
        })
    }

    /// Retrieve the internal `SocketAddr` of the listener.
    pub fn addr(&self) -> SocketAddr {
        self.listener
            .local_addr()
            .expect("Could not get bound address")
    }

    fn sock(&self) -> io::Result<&Sock> {
        self.sock
            .get_or_try_init::<_, io::Error>(|| Ok(Sock::new(self.listener.accept()?.0)))
    }

    fn sock_mut(&mut self) -> io::Result<&mut Sock> {
        self.sock()?;
        // Safe to unwrap, because we hold an exclusive
        // reference to `self`, and have just done a get_or_init call
        Ok(self.sock.get_mut().unwrap())
    }
}

impl Tubeable for Listen {
    fn get_receiver(&self) -> std::sync::mpsc::Receiver<Vec<u8>> {
        todo!()
    }

    fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        todo!()
    }
}
