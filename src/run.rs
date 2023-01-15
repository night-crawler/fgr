use std::io::{LineWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use ignore::{DirEntry, WalkState};
use nnf::parse_tree::ExpressionNode;

use crate::{Evaluate, GenericError};
use crate::parse::filter::Filter;

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
}

impl<T: Write> LineWriterExt for LineWriter<T> {
    fn write_line(&mut self, buf: impl AsRef<[u8]>) -> Result<(), std::io::Error> {
        self.write_all(buf.as_ref())?;
        self.write_all(b"\n")?;
        Ok(())
    }
}

pub fn spawn_receiver(
    status: &Arc<Mutex<ProcessStatus>>,
    receiver: kanal::Receiver<EntryMessage>,
) -> JoinHandle<i32> {
    let status = Arc::clone(status);

    std::thread::spawn(move || {
        let mut stdout = LineWriter::with_capacity(1024 * 16, std::io::stdout());
        let mut stderr = LineWriter::with_capacity(1024 * 16, std::io::stderr());

        loop {
            if !status.lock().unwrap().eq(&ProcessStatus::InProgress) {
                break 1;
            }

            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(EntryMessage::Success(entry)) => {
                    let write_result =
                        stdout.write_line(entry.path().to_string_lossy().as_bytes());
                    if write_result.is_err() {
                        let _ = stderr.write_line("Failed to write to stdout");
                        *status.lock().unwrap() = ProcessStatus::SendError;
                    }
                }
                Ok(EntryMessage::Init) => {
                    stdout.flush().unwrap();
                }
                Ok(EntryMessage::Error(entry, error)) => {
                    let _ = stderr.write_line(entry.path().to_string_lossy().as_bytes());
                    let _ = stderr.write_line(format!("\t{:?}", error));
                }
                Err(kanal::ReceiveErrorTimeout::Timeout) => {
                    let _ = stdout.flush();
                    let _ = stderr.flush();
                }
                Err(_) => {
                    break 0;
                }
            };
        }
    })
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
