use crate::tubes::buffer::Buffer;
use crate::tubes::tube::Tube;
use crate::Tubeable;
use nix::pty::openpty;
use nix::unistd::dup;

use std::fs::File;
use std::io::{self, prelude::*, BufReader, BufWriter};
use std::os::fd::{FromRawFd, OwnedFd};
use std::process::{Child, Stdio};

pub struct Process {
    child: Child,
    reader: BufReader<File>,
    writer: BufWriter<File>,
    buffer: Buffer,
}

impl Process {
    pub fn new<'a>(command: impl Into<&'a str>, shell: Option<&'a str>) -> Result<Self, io::Error> {
        let command: &str = command.into();
        let pty_pair = openpty(None, None)?;
        unsafe {
            let child = std::process::Command::new(shell.unwrap_or("/bin/bash"))
                .arg("-c")
                .arg(command)
                .stdin(Stdio::from_raw_fd(pty_pair.slave))
                .stderr(Stdio::from_raw_fd(pty_pair.slave))
                .stdout(Stdio::from_raw_fd(pty_pair.slave))
                .spawn()?;
            drop(OwnedFd::from_raw_fd(pty_pair.slave));
            let writer = BufWriter::new(File::from_raw_fd(dup(pty_pair.master)?));
            let reader = BufReader::new(File::from_raw_fd(pty_pair.master));
            Ok(Process {
                child,
                reader,
                writer,
                buffer: Buffer::new(),
            })
        }
    }
}

impl Tubeable for Process {
    fn get_buffer(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    fn fill_buffer(&mut self, timeout: Option<std::time::Duration>) -> io::Result<usize> {
        let mut temp_buf: [u8; 1024] = [0; 1024];
        let mut total: usize = 0;
        loop {
            let read = self.reader.read(&mut temp_buf)?;
            let buffer = self.get_buffer();
            buffer.add(temp_buf[..read].to_vec());
            total += read;
            if read < 1024 {
                break;
            }
        }
        Ok(total)
    }

    fn send_raw(&mut self, data: Vec<u8>) -> io::Result<()> {
        self.writer.write_all(data.as_slice())
    }

    fn close(&mut self) -> io::Result<()> {
        todo!()
    }
}
