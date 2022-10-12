use std::borrow::BorrowMut;
use std::sync::Arc;
use std::thread::JoinHandle;
use ignore::{DirEntry, Error, WalkState};
use kanal::{Receiver, Sender};
use crate::{Evaluate, ExpressionNode, GenericError};

#[derive(Eq, PartialEq)]
pub enum ProcessStatus {
    InProgress,
    SendError,
    Cancelled,
}

#[derive(Debug)]
pub struct EntryMessage {
    dir_entry: DirEntry,
}

impl EntryMessage {
    fn new(dir_entry: DirEntry) -> EntryMessage {
        Self { dir_entry }
    }
}

pub fn spawn_senders(
    status: &Arc<ProcessStatus>,
    root_node: &Arc<ExpressionNode>,
    sender: Sender<EntryMessage>,
    parallel_walker: ignore::WalkParallel,
) {
    parallel_walker.run(|| {
        let root = Arc::clone(root_node);
        let mut status = Arc::clone(status);
        let sender = sender.clone();

        Box::new(move |entry| {
            if status.as_ref() != &ProcessStatus::InProgress {
                return WalkState::Quit;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => {
                    return WalkState::Continue;
                }
            };

            match root.evaluate(&entry) {
                Ok(matched) => {
                    if matched {
                        let entry = EntryMessage::new(entry);
                        if sender.send(entry).is_err() {
                            *status.borrow_mut() = Arc::from(ProcessStatus::SendError);
                            return WalkState::Quit;
                        }
                    }
                }
                Err(GenericError::IgnoreError(Error::WithPath { .. })) => {}
                Err(_) => {}
            }
            WalkState::Continue
        })
    })
}

pub fn spawn_receiver(
    status: &Arc<ProcessStatus>,
    receiver: Receiver<EntryMessage>,
) -> JoinHandle<i32> {
    let status = Arc::clone(status);
    std::thread::spawn(move || {
        loop {
            if status.as_ref() != &ProcessStatus::InProgress {
                break 1;
            }

            match receiver.recv() {
                Ok(entry) => {
                    println!("{}", entry.dir_entry.path().to_string_lossy())
                }
                Err(_) => {
                    break 0;
                }
            };
        }
    })
}

pub fn set_int_handler(status: &Arc<ProcessStatus>) {
    let mut status = Arc::clone(status);
    ctrlc::set_handler(move || {
        if status.as_ref() == &ProcessStatus::Cancelled {
            std::process::exit(130);
        }
        *status.borrow_mut() = Arc::from(ProcessStatus::Cancelled);
    })
        .unwrap();
}
