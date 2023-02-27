#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

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

    impl MyAsyncWorker {
        pub fn new(initial_value: f32) -> Self {
            Self {
                current_value: Arc::new(Mutex::new(initial_value)),
            }
        }
        pub fn add(&mut self, value: f32) -> f32 {
            let mut guard = self.current_value.lock();
            *guard += value;
            *guard
        }
        pub fn divide(&mut self, divisor: f32) -> Result<f32, MyAsyncWorkerError> {
            if divisor == 0.0 {
                Err(MyAsyncWorkerError::DivisionByZero)
            } else {
                let mut guard = self.current_value.lock();
                *guard /= divisor;
                Ok(*guard)
            }
        }
        pub fn noop(&mut self) {}
    }
    impl MyAsyncWorker {
        pub fn execute_command(&mut self, command: MyAsyncWorkerCommand) {
            match command {
                MyAsyncWorkerCommand::Add {
                    result_sender,
                    value,
                } => {
                    let ret = self.add(value);
                    let _ = result_sender.send(ret);
                }
                MyAsyncWorkerCommand::Divide {
                    result_sender,
                    divisor,
                } => {
                    let ret = self.divide(divisor);
                    let _ = result_sender.send(ret);
                }
                MyAsyncWorkerCommand::Noop { result_sender } => {
                    let ret = self.noop();
                    let _ = result_sender.send(ret);
                }
            }
        }
        pub fn try_execute_task(
            &mut self,
            receiver: &mut async_worker_fn::command_channel::CommandReceiver<MyAsyncWorkerCommand>,
        ) -> Result<bool, async_worker_fn::AllClientsDroppedError> {
            let command = receiver.try_recv();
            match command {
                Ok(command) => {
                    self.execute_command(command);
                    return Ok(true);
                }
                Err(async_worker_fn::command_channel::TryRecvError::Empty) => return Ok(false),
                Err(async_worker_fn::command_channel::TryRecvError::Disconnected) => {
                    return Err(async_worker_fn::AllClientsDroppedError)
                }
            }
        }
        pub fn execute_command_queue(
            &mut self,
            receiver: &mut async_worker_fn::command_channel::CommandReceiver<MyAsyncWorkerCommand>,
        ) -> Result<(), async_worker_fn::AllClientsDroppedError> {
            while self.try_execute_task(receiver)? {}
            Ok(())
        }
    }
    pub enum MyAsyncWorkerCommand {
        Add {
            value: f32,
            result_sender: ::tokio::sync::oneshot::Sender<f32>,
        },
        Divide {
            divisor: f32,
            result_sender: ::tokio::sync::oneshot::Sender<Result<f32, MyAsyncWorkerError>>,
        },
        Noop {
            result_sender: ::tokio::sync::oneshot::Sender<()>,
        },
    }
    fn my_async_worker_channel() -> (
        async_worker_fn::command_channel::CommandSender<MyAsyncWorkerCommand>,
        async_worker_fn::command_channel::CommandReceiver<MyAsyncWorkerCommand>,
    ) {
        async_worker_fn::command_channel::command_channel()
    }
    #[derive(Clone)]
    pub struct MyAsyncWorkerClient {
        command_sender: async_worker_fn::command_channel::CommandSender<MyAsyncWorkerCommand>,
    }
    impl MyAsyncWorkerClient {
        fn new(
            sender: async_worker_fn::command_channel::CommandSender<MyAsyncWorkerCommand>,
        ) -> Self {
            Self {
                command_sender: sender,
            }
        }
        pub fn add(
            &self,
            value: f32,
        ) -> impl ::std::future::Future<Output = Result<f32, async_worker_fn::AllWorkersDroppedError>>
        {
            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
            let ret = self.command_sender.send(MyAsyncWorkerCommand::Add {
                result_sender,
                value,
            });
            async move {
                if let Err(e) = ret {
                    ::log::error!("MyAsyncWorkerClient::add, msg = {:?}", e);
                    return Err(async_worker_fn::AllWorkersDroppedError);
                }
                match result_receiver.await {
                    Ok(ret) => Ok(ret),
                    Err(e) => {
                        ::log::error!("MyAsyncWorkerClient::add response, msg = {:?}", e);
                        Err(async_worker_fn::AllWorkersDroppedError)
                    }
                }
            }
        }
        pub fn divide(
            &self,
            divisor: f32,
        ) -> impl ::std::future::Future<
            Output = Result<
                Result<f32, MyAsyncWorkerError>,
                async_worker_fn::AllWorkersDroppedError,
            >,
        > {
            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
            let ret = self.command_sender.send(MyAsyncWorkerCommand::Divide {
                result_sender,
                divisor,
            });
            async move {
                if let Err(e) = ret {
                    ::log::error!("MyAsyncWorkerClient::divide, msg = {:?}", e);
                    return Err(async_worker_fn::AllWorkersDroppedError);
                }
                match result_receiver.await {
                    Ok(ret) => Ok(ret),
                    Err(e) => {
                        ::log::error!("MyAsyncWorkerClient::divide response, msg = {:?}", e);
                        Err(async_worker_fn::AllWorkersDroppedError)
                    }
                }
            }
        }
        pub fn noop(
            &self,
        ) -> impl ::std::future::Future<Output = Result<(), async_worker_fn::AllWorkersDroppedError>>
        {
            let (result_sender, result_receiver) = tokio::sync::oneshot::channel();
            let ret = self
                .command_sender
                .send(MyAsyncWorkerCommand::Noop { result_sender });
            async move {
                if let Err(e) = ret {
                    ::log::error!("MyAsyncWorkerClient::noop, msg = {:?}", e);
                    return Err(async_worker_fn::AllWorkersDroppedError);
                }
                match result_receiver.await {
                    Ok(ret) => Ok(ret),
                    Err(e) => {
                        ::log::error!("MyAsyncWorkerClient::noop response, msg = {:?}", e);
                        Err(async_worker_fn::AllWorkersDroppedError)
                    }
                }
            }
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn caling_async_worker_fn_directly() {
        let mut worker = MyAsyncWorker::new(7.0);
        assert_eq!(*worker.current_value.lock(), 7.0);

        let ret = worker.divide(2.0).unwrap();
        assert_eq!(*worker.current_value.lock(), 3.5);
        assert_eq!(ret, 3.5);

        let ret = worker.add(15.0);
        assert_eq!(*worker.current_value.lock(), 18.5);
        assert_eq!(ret, 18.5);

        worker.noop();
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
