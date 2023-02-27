use std::{
    collections::VecDeque,
    sync::{
        atomic::{self, AtomicUsize},
        Arc,
    },
};

use parking_lot::Mutex;
use tokio::sync::watch;

use crate::prelude::ArcMutex;

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

struct Shared<T: Send> {
    queue: ArcMutex<VecDeque<T>>,
    sender_count: Arc<AtomicUsize>,
    // todo!("use an async condvar")
    queue_watcher_sender: Arc<watch::Sender<()>>,
}

pub struct CommandSender<T: Send> {
    shared: Shared<T>,
}

pub struct CommandReceiver<T: Send> {
    shared: Shared<T>,
    queue_watcher_receiver: watch::Receiver<()>,
}

pub fn command_channel<T: Send>() -> (CommandSender<T>, CommandReceiver<T>) {
    let (sender, receiver) = watch::channel(());

    let shared = Shared {
        queue: Arc::new(Mutex::new(VecDeque::new())),
        sender_count: Arc::new(AtomicUsize::new(1)),
        queue_watcher_sender: Arc::new(sender),
    };

    (
        CommandSender {
            shared: shared.clone(),
        },
        CommandReceiver {
            shared,
            queue_watcher_receiver: receiver,
        },
    )
}

impl<T: Send> CommandSender<T> {
    pub fn send(&self, command: T) -> Result<(), SendError> {
        if self.shared.queue_watcher_sender.receiver_count() != 0 {
            self.shared.queue.lock().push_back(command);
            let _ = self.shared.queue_watcher_sender.send(());

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
                if self.shared.sender_count.load(atomic::Ordering::SeqCst) == 0 {
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
        let mut queue_guard = self.shared.queue.lock();
        if let Some(command) = queue_guard.pop_front() {
            let _ = self.shared.queue_watcher_sender.send(());
            Ok(command)
        } else if self.shared.sender_count.load(atomic::Ordering::SeqCst) == 0 {
            Err(TryRecvError::Disconnected)
        } else {
            Err(TryRecvError::Empty)
        }
    }
}

impl<T: Send> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            queue: self.queue.clone(),
            sender_count: self.sender_count.clone(),
            queue_watcher_sender: self.queue_watcher_sender.clone(),
        }
    }
}

impl<T: Send> Clone for CommandSender<T> {
    fn clone(&self) -> Self {
        self.shared
            .sender_count
            .fetch_add(1, atomic::Ordering::SeqCst);
        Self {
            shared: self.shared.clone(),
        }
    }
}

impl<T: Send> Drop for CommandSender<T> {
    fn drop(&mut self) {
        self.shared
            .sender_count
            .fetch_sub(1, atomic::Ordering::SeqCst);

        let _ = self.shared.queue_watcher_sender.send(());
    }
}

impl<T: Send> Clone for CommandReceiver<T> {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            queue_watcher_receiver: self.queue_watcher_receiver.clone(),
        }
    }
}

impl<T: Send> Drop for CommandReceiver<T> {
    fn drop(&mut self) {
        // if this is the last receiver, then empty the queue
        if self.shared.queue_watcher_sender.receiver_count() == 1 {
            let mut queue_guard = self.shared.queue.lock();
            if !queue_guard.is_empty() {
                queue_guard.clear();
                let _ = self.shared.queue_watcher_sender.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use parking_lot::Mutex;

    use crate::prelude::ArcMutex;

    use super::{command_channel, CommandReceiver};

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    struct Command(usize);

    async fn run_worker(
        received_values: ArcMutex<Vec<Command>>,
        mut receiver: CommandReceiver<Command>,
    ) {
        while let Ok(command) = receiver.recv_async().await {
            received_values.lock().push(command);
        }
    }

    async fn run_test(number_of_workers: usize) {
        let received_values = Arc::new(Mutex::new(Vec::<Command>::new()));

        let (sender, receiver) = command_channel();

        let mut workers = Vec::new();
        for _ in 0..number_of_workers {
            let received_values = received_values.clone();
            let receiver = receiver.clone();
            workers.push(tokio::spawn(run_worker(received_values, receiver)));
        }

        sender.send(Command(7)).unwrap();

        drop(sender);

        for worker in workers {
            worker.await.unwrap();
        }

        let received_values = received_values.lock();
        assert_eq!(received_values.len(), 1);
        assert_eq!(*received_values.get(0).unwrap(), Command(7));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn single_worker_current_thread() {
        // running the same test multiple times to ensure no race condition happened
        for _i in 0..1000 {
            run_test(1).await;
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn multiple_workers_current_thread() {
        // running the same test multiple times to ensure no race condition happened
        for _i in 0..1000 {
            run_test(10).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn single_worker_multi_thread() {
        // running the same test multiple times to ensure no race condition happened
        for _i in 0..1000 {
            run_test(1).await;
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn multiple_workers_multi_thread() {
        // running the same test multiple times to ensure no race condition happened
        for _i in 0..1000 {
            run_test(10).await;
        }
    }
}
