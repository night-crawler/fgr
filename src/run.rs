use std::io::{LineWriter, Stderr, Stdout, Write};
use std::os::unix::ffi::OsStrExt;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use ignore::{DirEntry, WalkState};
use nnf::parse_tree::ExpressionNode;

use crate::config::Config;
use crate::parse::filter::Filter;
use crate::{Evaluate, GenericError};

#[derive(Eq, PartialEq)]
pub enum ProcessStatus {
    InProgress,
    SendError,
    Cancelled,
}

#[derive(Debug)]
pub enum EntryMessage {
    Success(DirEntry),
    Error(DirEntry, GenericError),
    Init,
}

pub fn spawn_senders(
    status: &Arc<Mutex<ProcessStatus>>,
    root_node: &Arc<ExpressionNode<Filter>>,
    sender: kanal::Sender<EntryMessage>,
    parallel_walker: ignore::WalkParallel,
) {
    parallel_walker.run(|| {
        let root = Arc::clone(root_node);
        let status = Arc::clone(status);
        let sender = sender.clone();

        sender.send(EntryMessage::Init).unwrap();

        Box::new(move |entry| {
            if !status.lock().unwrap().eq(&ProcessStatus::InProgress) {
                return WalkState::Quit;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    return WalkState::Continue;
                }
            };

            let eval_result = root.evaluate(&entry);

            let message = match eval_result {
                Ok(matched) if matched => EntryMessage::Success(entry),
                Err(error) => match &error {
                    GenericError::IoError(io_error)
                        if io_error.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        EntryMessage::Error(entry, error)
                    }
                    _ => return WalkState::Continue,
                },
                _ => return WalkState::Continue,
            };

            if sender.send(message).is_err() {
                *status.lock().unwrap() = ProcessStatus::SendError;
                return WalkState::Quit;
            }

            WalkState::Continue
        })
    })
}

trait LineWriterExt {
    fn write_line(&mut self, buf: impl AsRef<[u8]>) -> Result<(), std::io::Error>;
    fn write_line_sep(
        &mut self,
        buf: impl AsRef<[u8]>,
        sep: u8,
    ) -> Result<(), std::io::Error>;
}

impl<T: Write> LineWriterExt for LineWriter<T> {
    #[inline(always)]
    fn write_line(&mut self, buf: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        self.write_line_sep(buf, b'\n')
    }

    #[inline(always)]
    fn write_line_sep(
        &mut self,
        buf: impl AsRef<[u8]>,
        sep: u8,
    ) -> Result<(), std::io::Error> {
        self.write_all(buf.as_ref())?;
        self.write_all(&[sep])?;
        Ok(())
    }
}

pub struct EntryReceiver {
    status: Arc<Mutex<ProcessStatus>>,
    receiver: kanal::Receiver<EntryMessage>,
    stdout: LineWriter<Stdout>,
    stderr: LineWriter<Stderr>,
    recv_timeout: Duration,
    separator: u8,
}

impl EntryReceiver {
    pub fn new(
        config: Config,
        stdout_capacity: usize,
        stderr_capacity: usize,
        receiver: kanal::Receiver<EntryMessage>,
        recv_timeout: Duration,
        status: &Arc<Mutex<ProcessStatus>>,
    ) -> Self {
        let stdout = LineWriter::with_capacity(stdout_capacity, std::io::stdout());
        let stderr = LineWriter::with_capacity(stderr_capacity, std::io::stderr());

        let separator = if config.print0 { b'\0' } else { b'\n' };

        Self {
            separator,
            stdout,
            stderr,
            receiver,
            recv_timeout,
            status: Arc::clone(status),
        }
    }

    fn receive(&mut self) -> Result<(), kanal::ReceiveErrorTimeout> {
        match self.receiver.recv_timeout(self.recv_timeout) {
            Ok(EntryMessage::Success(entry)) => {
                // write the name without converting it to utf8
                let write_result = self
                    .stdout
                    .write_line_sep(entry.path().as_os_str().as_bytes(), self.separator);

                if write_result.is_err() {
                    let _ = self.stderr.write_line("Failed to write to stdout");
                    *self.status.lock().unwrap() = ProcessStatus::SendError;
                }
            }
            Ok(EntryMessage::Init) => {
                self.stdout.flush().unwrap();
            }
            Ok(EntryMessage::Error(entry, error)) => {
                // write the name without converting it to utf8
                let _ = self.stderr.write_line(entry.path().as_os_str().as_bytes());
                let _ = self.stderr.write_line(format!("\t{:?}", error));
            }
            Err(kanal::ReceiveErrorTimeout::Timeout) => {
                let _ = self.stdout.flush();
                let _ = self.stderr.flush();
            }
            Err(err) => {
                return Err(err);
            }
        }

        Ok(())
    }

    pub fn receive_all(mut self) -> JoinHandle<i32> {
        std::thread::spawn(move || {
            loop {
                if !self.status.lock().unwrap().eq(&ProcessStatus::InProgress) {
                    break 1;
                }

                // TODO: check for other errors
                if self.receive().is_err() {
                    break 0;
                }
            }
        })
    }
}

pub fn set_int_handler(status: &Arc<Mutex<ProcessStatus>>) {
    let status = Arc::clone(status);
    ctrlc::set_handler(move || {
        if status.lock().unwrap().eq(&ProcessStatus::Cancelled) {
            std::process::exit(130);
        }

        *status.lock().unwrap() = ProcessStatus::Cancelled;
    })
    .unwrap();
}
