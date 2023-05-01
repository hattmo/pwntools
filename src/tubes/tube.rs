use crate::debug;
use crossbeam_utils::thread;
use regex::bytes::{Regex, RegexBuilder};
use rustyline::Editor;
use std::fmt::Display;
use std::io;
use std::io::prelude::*;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};
pub struct Tube<T>
where
    T: Tubeable + Send + Sync,
{
    tube: T,
    receiver: Receiver<Vec<u8>>,
    buffer: Vec<u8>,
    timeout: Duration,
}

impl<T> Deref for Tube<T>
where
    T: Tubeable + Send + Sync,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.tube
    }
}

impl<T> DerefMut for Tube<T>
where
    T: Tubeable + Send + Sync,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tube
    }
}

impl<T> Tube<T>
where
    T: Tubeable + Send + Sync,
{
    fn new(tube: T) -> Tube<T> {
        Tube {
            tube,
            receiver: tube.get_receiver(),
            buffer: Vec::with_capacity(1024),
            timeout: Duration::from_secs(1),
        }
    }

    fn fill_buffer(&mut self, ammount: Option<usize>, timeout: Option<Duration>) {
        let dead_line = Instant::now() + timeout.unwrap_or(Duration::ZERO);
        while let Ok(ref mut chunk) = self.tube.get_receiver().recv_deadline(dead_line) {
            self.buffer.extend(chunk.iter());
        }
    }

    pub fn clean(&mut self) {
        self.fill_buffer(None, None);
        self.buffer.clear();
    }

    // pub fn clean(&mut self, timeout: Duration) -> io::Result<Vec<u8>> {
    //     self.fill_buffer(Some(timeout));
    //     Ok(self.buffer.get(0))
    // }

    /// Receives from the `Tube`, returning once any data is available.
    pub fn recv(&mut self) -> Vec<u8> {
        self.fill_buffer(None, None);
        if self.buffer.len() == 0 {
            self.fill_buffer(None, Some(self.timeout));
        }
        std::mem::replace(&mut self.buffer,Vec::with_capacity(1024))
    }

    /// Receives `n` bytes from the `Tube`.
    fn recvn(&mut self, n: usize) -> Result<Vec<u8>, ()> {
        self.fill_buffer(Some(n), None);
        let mut out = Vec::with_capacity(n);
        if self.buffer.len() >= n {
            self.buffer.read_exact(out.as_mut());
            return Ok(out);
        }
        self.fill_buffer(Some(n), Some(self.timeout));
        if self.buffer.len() >= n {
            self.buffer.read_exact(out.as_mut());
            return Ok(out);
        }
        Err(())
    }

    /// Receive until the given delimiter is received.
    fn recvuntil(&mut self, pattern: &str) -> io::Result<Vec<u8>> {
        let regex = Regex::new(pattern).or(Err(io::Error::other("Invalid Regex")))?;
        loop {
            let buf = self.buffer.make_contiguous();
            if let Some(found) = regex.find(buf) {
                return Ok(self.buffer.split_off(found.end()).into());
            }
            self.fill_buffer(None, Some(self.timeout));
        }
    }

    /// Receive from the tube until a newline is received.
    fn recvline(&mut self) -> io::Result<Vec<u8>> {
        self.recvuntil("\n")
    }

    fn recvall(&mut self) -> Result<Vec<u8>, ()> {
        Ok(vec![])
    }
    /// Writes data to the `Tube`.
    fn send<V: Into<Vec<u8>>>(&mut self, data: V) -> io::Result<()> {
        let data = data.into();
        debug!("Sending {} bytes", data.len());
        self.tube.send(data)
    }
    /// Appends a newline to the data before writing it to the `Tube`.
    fn sendline<V: Into<Vec<u8>>>(&mut self, data: V) -> io::Result<()> {
        let mut data = data.into();
        data.push(b'\n');
        debug!("Sending {} bytes", data.len());
        self.tube.send(data)
    }

    /// Get an interactive prompt for the connection. A second thread will print messages as they
    /// arrive.
    pub fn interactive(self) -> io::Result<()>
    where
        Self: Clone + Send,
    {
        let Tube { buffer, tube, .. } = self;

        let in_job = thread::spawn(move || loop {
            let mut rl = Editor::<(), FileHistory>::new().unwrap();

            while let Ok(line) = rl.readline("$ ") {
                if tube.send(line.into_bytes()).is_err() {
                    return;
                }
            }
        });
        let receiver = self.tube.get_receiver();

        let mut stdout = std::io::stdout();
        io::copy(&mut self.buffer, &mut stdout);
        while let Some(chunk) = receiver.iter().next() {
            stdout.write_all(&chunk);
        }
        in_job.join().unwrap();
        Ok(())
    }
}

pub trait Tubeable {
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
    fn send(&mut self, data: Vec<u8>) -> io::Result<()>;
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

struct PrettyBytes<'a>(&'a [u8]);

impl<'a> Display for PrettyBytes<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.into_iter().map(|b| {
            if b.is_ascii_graphic() {
                return String::from(char::from(*b));
            }
            format!("{b:2.2x}")
        });
        write!(f, "Hello")
    }
}
#[test]
fn test() {
    let foo = PrettyBytes(b"hello");
    println!("{}", foo);
}
