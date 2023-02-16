use crate::tubes::buffer::Buffer;
use crate::tubes::tube::Tube;
use nix::pty::openpty;
use nix::unistd::dup;

use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::os::fd::{FromRawFd, OwnedFd};
use std::process::{Child, Stdio};

pub struct Process {
    child: Child,
    reader: BufReader<File>,
    writer: BufWriter<File>,
    buffer: Buffer,
}

impl Process {
    pub fn new<'a>(command: impl Into<&'a str>) -> Result<Self, io::Error> {
        let command: &str = command.into();
        let pty_pair = openpty(None, None)?;
        unsafe {
            let child = std::process::Command::new("/bin/bash")
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

#[cfg(test)]
mod test {
    use std::io::prelude::*;

    use crate::Process;

    #[test]
    fn test() {
        let proc = Process::new("whoami").expect("fail");

        for line in proc.reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(_) => {
                    break;
                }
            }
        }
    }
}

impl Tube for Process {
    fn get_buffer(&mut self) -> &mut Buffer {
        todo!()
    }

    fn fill_buffer(&mut self, timeout: Option<std::time::Duration>) -> io::Result<usize> {}

    fn send_raw(&mut self, data: Vec<u8>) -> io::Result<()> {
        todo!()
    }

    fn close(&mut self) -> io::Result<()> {
        todo!()
    }
}
