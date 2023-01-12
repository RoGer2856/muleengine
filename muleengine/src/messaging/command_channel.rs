use std::{
    collections::VecDeque,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};

use tokio::sync::watch;

#[derive(Debug)]
pub enum SendError {
    Disconnected,
}

#[derive(Debug)]
pub enum RecvError {
    Disconnected,
}

#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

pub struct CommandSender<T: Send> {
    sender_count: Arc<AtomicUsize>,
    queue_watcher_sender: Arc<watch::Sender<VecDeque<T>>>,
}

pub struct CommandReceiver<T: Send> {
    sender_count: Arc<AtomicUsize>,
    queue_watcher_sender: Arc<watch::Sender<VecDeque<T>>>,
    queue_watcher_receiver: watch::Receiver<VecDeque<T>>,
}

pub fn command_channel<T: Send>() -> (CommandSender<T>, CommandReceiver<T>) {
    let (sender, receiver) = watch::channel(VecDeque::new());
    let sender = Arc::new(sender);

    let sender_count = Arc::new(AtomicUsize::new(1));

    (
        CommandSender {
            sender_count: sender_count.clone(),
            queue_watcher_sender: sender.clone(),
        },
        CommandReceiver {
            sender_count,
            queue_watcher_sender: sender,
            queue_watcher_receiver: receiver,
        },
    )
}

impl<T: Send> CommandSender<T> {
    pub fn send(&self, command: T) -> Result<(), SendError> {
        if self.queue_watcher_sender.receiver_count() != 0 {
            self.queue_watcher_sender.send_modify(|queue| {
                queue.push_back(command);
            });

            Ok(())
        } else {
            Err(SendError::Disconnected)
        }
    }
}

impl<T: Send> CommandReceiver<T> {
    pub async fn recv_async(&mut self) -> Result<T, RecvError> {
        loop {
            if self.queue_watcher_receiver.changed().await.is_ok() {
                match self.try_pop() {
                    Ok(command) => break Ok(command),
                    Err(TryRecvError::Disconnected) => break Err(RecvError::Disconnected),
                    Err(TryRecvError::Empty) => (),
                }
            } else {
                // unreachable!("This could not happen, since Self also holds a clone of the sender part");
                break Err(RecvError::Disconnected);
            }
        }
    }

    pub fn try_recv(&self) -> Result<T, TryRecvError> {
        match self.queue_watcher_receiver.has_changed() {
            Ok(true) => self.try_pop(),
            Ok(false) => {
                if self.sender_count.load(atomic::Ordering::SeqCst) == 0 {
                    Err(TryRecvError::Disconnected)
                } else {
                    Err(TryRecvError::Empty)
                }
            }
            Err(_) => {
                // unreachable!("This could not happen, since Self also holds a clone of the sender part");
                Err(TryRecvError::Disconnected)
            }
        }
    }

    pub fn try_pop(&self) -> Result<T, TryRecvError> {
        let mut ret = Err(TryRecvError::Empty);

        self.queue_watcher_sender.send_modify(|queue| {
            if let Some(command) = queue.pop_front() {
                ret = Ok(command);
            } else {
                ret = Err(TryRecvError::Empty);
            }
        });

        if let Err(TryRecvError::Empty) = ret {
            if self.sender_count.load(atomic::Ordering::SeqCst) == 0 {
                ret = Err(TryRecvError::Disconnected)
            }
        }

        ret
    }
}

impl<T: Send> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        self.sender_count.fetch_add(1, atomic::Ordering::SeqCst);
        Self {
            sender_count: self.sender_count.clone(),
            queue_watcher_sender: self.queue_watcher_sender.clone(),
        }
    }
}

impl<T: Send> Drop for CommandSender<T> {
    fn drop(&mut self) {
        // if this is the last sender, then notify all receivers
        if self.sender_count.fetch_sub(1, atomic::Ordering::SeqCst) == 1 {
            self.queue_watcher_sender.send_modify(|_queue| {});
        }
    }
}

impl<T: Send> Clone for CommandReceiver<T> {
    fn clone(&self) -> Self {
        Self {
            sender_count: self.sender_count.clone(),
            queue_watcher_sender: self.queue_watcher_sender.clone(),
            queue_watcher_receiver: self.queue_watcher_receiver.clone(),
        }
    }
}

impl<T: Send> Drop for CommandReceiver<T> {
    fn drop(&mut self) {
        // if this is the last receiver, then empty the queue
        if self.queue_watcher_sender.receiver_count() == 1 {
            self.queue_watcher_sender.send_modify(|queue| {
                queue.clear();
            });
        }
    }
}
