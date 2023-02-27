#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use async_worker_macros::async_worker_impl;
    use parking_lot::Mutex;
    use tokio::time::{sleep_until, Instant};

    use crate::{self as async_worker_fn, prelude::ArcMutex};

    #[derive(Debug)]
    pub enum MyAsyncWorkerError {
        DivisionByZero,
    }

    #[derive(Clone)]
    struct MyAsyncWorker {
        current_value: ArcMutex<f32>,
    }

    #[async_worker_impl(
        client_name = MyAsyncWorkerClient,
        channel_creator_fn = my_async_worker_channel,
        command_type = MyAsyncWorkerCommand,
    )]
    impl MyAsyncWorker {
        pub fn new(initial_value: f32) -> Self {
            Self {
                current_value: Arc::new(Mutex::new(initial_value)),
            }
        }

        #[async_worker_fn]
        pub fn add(&mut self, value: f32) -> f32 {
            let mut guard = self.current_value.lock();
            *guard += value;
            *guard
        }

        #[async_worker_fn]
        pub fn divide(&mut self, divisor: f32) -> Result<f32, MyAsyncWorkerError> {
            if divisor == 0.0 {
                Err(MyAsyncWorkerError::DivisionByZero)
            } else {
                let mut guard = self.current_value.lock();
                *guard /= divisor;
                Ok(*guard)
            }
        }

        #[async_worker_fn]
        pub fn noop(&mut self) {}
    }

    #[tokio::test(flavor = "current_thread")]
    async fn caling_async_worker_fn_directly() {
        let mut async_worker = MyAsyncWorker::new(7.0);
        assert_eq!(*async_worker.current_value.lock(), 7.0);

        let ret = async_worker.divide(2.0).unwrap();
        assert_eq!(*async_worker.current_value.lock(), 3.5);
        assert_eq!(ret, 3.5);

        let ret = async_worker.add(15.0);
        assert_eq!(*async_worker.current_value.lock(), 18.5);
        assert_eq!(ret, 18.5);

        async_worker.noop();
    }

    #[tokio::test(flavor = "current_thread")]
    async fn single_async_worker() {
        let (sender, mut receiver) = my_async_worker_channel();
        let client = MyAsyncWorkerClient::new(sender);
        let mut worker = MyAsyncWorker::new(7.0);

        let client_task = {
            let worker = worker.clone();
            tokio::spawn(async move {
                let ret = client.divide(2.0).await.unwrap().unwrap();
                assert_eq!(*worker.current_value.lock(), 3.5);
                assert_eq!(ret, 3.5);

                let ret = client.add(15.0).await.unwrap();
                assert_eq!(*worker.current_value.lock(), 18.5);
                assert_eq!(ret, 18.5);

                client.noop().await.unwrap();
            })
        };

        tokio::spawn(async move {
            while let Ok(command) = receiver.recv_async().await {
                let _ = worker.execute_command(command);
            }
        });

        let timeout = Duration::from_secs(5);
        let timestamp = Instant::now() + timeout;
        tokio::select! {
            _ = sleep_until(timestamp) => {
                assert!(false, "timed out");
            }
            ret = client_task => {
                ret.unwrap();
            }
        }
    }
}
