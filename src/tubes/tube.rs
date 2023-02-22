extern crate rustyline;

use rustyline::Editor;
use std::fmt::Display;
use std::io::Write;
use std::io::{self, Empty};

extern crate crossbeam_utils;
use crate::{debug, Buffer};
use crossbeam_utils::thread;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;
pub struct Tube<T>
where
    T: Tubeable,
{
    tube: T,
    receiver: Receiver<Vec<u8>>,
    buffer: Buffer,
}

impl<T> Tube<T>
where
    T: Tubeable,
{
    fn new(tube: T) -> Tube<T> {
        Tube {
            tube,
            receiver: tube.get_receiver(),
            buffer: Buffer::new(),
        }
    }

    fn fill_buffer(&mut self, timeout: Option<Duration>) {
        loop {
            match self
                .receiver
                .recv_timeout(timeout.unwrap_or(Duration::ZERO))
            {
                Ok(chunk) => self.buffer.add(chunk),
                Err(RecvTimeoutError::Timeout) => todo!(),
                Err(RecvTimeoutError::Disconnected) => todo!(),
                //
            }
        }
    }

    pub fn clean(&mut self, timeout: Duration) -> io::Result<Vec<u8>> {
        self.fill_buffer(Some(timeout));
        Ok(self.buffer.get(0))
    }

    /// Receives from the `Tube`, returning once any data is available.
    fn recv(&mut self) -> io::Result<Vec<u8>> {
        self.recv_raw(None, None)
    }
    /// Receives `n` bytes from the `Tube`.
    fn recvn(&mut self, n: usize) -> io::Result<Vec<u8>> {
        self.recv_raw(Some(n), None)
    }

    #[doc(hidden)]
    fn recv_raw(&mut self, numb: Option<usize>, timeout: Option<Duration>) -> io::Result<Vec<u8>> {
        self.fill_buffer(timeout)?;
        let numb = numb.unwrap_or(0);
        Ok(self.get_buffer().get(numb))
    }
    /// Receive all data from the `Tube`, repeatedly reading with the given `timeout`.
    fn recvrepeat(&mut self, timeout: Option<Duration>) -> io::Result<Vec<u8>> {
        while self.fill_buffer(timeout)? > 0 {}
        Ok(self.get_buffer().get(0))
    }
    /// Writes data to the `Tube`.
    fn send<V: Into<Vec<u8>>>(&mut self, data: V) -> io::Result<()> {
        let data = data.into();
        debug!("Sending {} bytes", data.len());
        self.send_raw(data)
    }
    /// Appends a newline to the data before writing it to the `Tube`.
    fn sendline<V: Into<Vec<u8>>>(&mut self, data: V) -> io::Result<()> {
        let mut data = data.into();
        data.push(b'\n');
        debug!("Sending {} bytes", data.len());
        self.send_raw(data)
    }

    /// Receive until the given delimiter is received.
    fn recvuntil(&mut self, delim: &[u8]) -> io::Result<Vec<u8>> {
        let mut pos;
        loop {
            self.fill_buffer(Some(Duration::from_millis(50)))?;
            pos = find_subsequence(self.get_buffer().data.make_contiguous(), delim);
            if let Some(p) = pos {
                return Ok(self.get_buffer().get(p + 1));
            }
        }
    }

    /// Receive from the tube until a newline is received.
    fn recvline(&mut self) -> io::Result<Vec<u8>> {
        self.recvuntil(b"\n")
    }
    /// Get an interactive prompt for the connection. A second thread will print messages as they
    /// arrive.
    pub fn interactive(&mut self) -> io::Result<()>
    where
        Self: Clone + Send,
    {
        let mut receiver = self.clone();
        // Make sure that the receiver thread does not outlive scope
        thread::scope(|s| {
            s.spawn(|_| loop {
                std::io::stdout()
                    .write_all(
                        &receiver
                            .clean(Duration::from_millis(50))
                            .unwrap_or_default(),
                    )
                    .expect("Couldn't write stdout")
            });

            let mut rl = Editor::<()>::new();
            loop {
                if let Ok(line) = rl.readline("$ ") {
                    if self.sendline(line).is_err() {
                        return;
                    }
                } else {
                    return;
                }
            }
        })
        .expect("Couldn't start receiving thread");
        Ok(())
    }
}

/// Generic `Tube` trait, used as the underlying interface for IO.
pub(crate) trait Tubeable {
    /// Retrieve mutable reference to the internal [`Buffer`].
    fn get_receiver(&self) -> Receiver<Vec<u8>>;
    /// Fill the internal [`Buffer`].
    ///
    /// * `timeout` - Maximum time to fill for. If `None`, block until data is read.

    // Currently not working. Gives a timeout message.
    /// Retrieve all data from the `Tube`.
    ///
    /// * `timeout` - The maximum time to read for, defaults to 0.05s. If 0, clean only the
    /// internal buffer.
    ///
    fn write(&mut self, data: Vec<u8>) -> io::Result<()>;
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

struct Bytes<'a>(&'a [u8]);

impl<'a> Display for Bytes<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let precision = f.precision().unwrap_or(20);

        write!(f, "Hello")
    }
}
#[test]
fn test() {
    let foo = b"hello";
    println!("{:?}", foo);
}
