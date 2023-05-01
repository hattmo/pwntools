use crate::Tubeable;
use nix::{pty::openpty, unistd::dup};
use std::{
    ffi::OsStr,
    fs::File,
    io::{self, prelude::*, BufReader, BufWriter},
    os::fd::{FromRawFd, OwnedFd},
    process::{Child, Stdio},
    sync::mpsc::{self, Receiver},
    thread::{spawn, JoinHandle},
};

pub struct Process {
    child: Child,
    recv: Receiver<Vec<u8>>,
    writer: BufWriter<File>,
    read_job: JoinHandle<()>,
}

impl Process {
    pub(crate) fn new<'a, T, U>(command: U, shell: Option<T>) -> Result<Self, io::Error>
    where
        U: AsRef<str>,
        T: AsRef<OsStr>,
    {
        let command = command.as_ref();
        let pty_pair = openpty(None, None)?;
        let shell = match shell {
            Some(s) => s,
            None => OsStr::new("/bin/sh").to_owned(),
        };
        unsafe {
            let child = std::process::Command::new(shell)
                .arg("-c")
                .arg(command)
                .stdin(Stdio::from_raw_fd(pty_pair.slave))
                .stderr(Stdio::from_raw_fd(pty_pair.slave))
                .stdout(Stdio::from_raw_fd(pty_pair.slave))
                .spawn()?;
            drop(OwnedFd::from_raw_fd(pty_pair.slave));
            let writer = BufWriter::new(File::from_raw_fd(dup(pty_pair.master)?));
            let reader = BufReader::new(File::from_raw_fd(pty_pair.master));
            let (send, recv) = mpsc::channel();
            let read_job = spawn(move || {
                let mut buf = [0u8; 256];
                while let Ok(num_read) = reader.read(buf.as_mut()) {
                    let out = Vec::with_capacity(num_read);
                    out.extend(&buf[..num_read]);
                    send.send(out);
                }
            });
            Ok(Process {
                child,
                recv,
                writer,
                read_job,
            })
        }
    }

    pub fn wait_for_exit(&self) -> Result<std::process::ExitStatus, io::Error> {
        self.child.wait()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.child.kill();
        self.read_job.join();
        self.child.wait();
    }
}

impl Tubeable for Process {
    fn get_receiver(&self) -> Receiver<Vec<u8>> {
        self.recv
    }

    fn send(&mut self, data: Vec<u8>) -> io::Result<()> {
        self.writer.write_all(data.as_ref())
    }
}
