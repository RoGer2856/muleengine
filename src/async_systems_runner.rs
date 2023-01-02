use std::future::Future;

use muleengine::prelude::ResultInspector;
use tokio::task::JoinHandle;

pub struct AsyncSystemsRunner {
    async_systems: JoinHandle<Vec<JoinHandle<()>>>,
}

impl AsyncSystemsRunner {
    pub fn run(
        systems_initializer: impl Future<Output = Vec<JoinHandle<()>>> + Send + 'static,
    ) -> Self {
        Self {
            async_systems: tokio::spawn(async move { systems_initializer.await }),
        }
    }

    pub async fn join(self) {
        let async_systems = self
            .async_systems
            .await
            .inspect_err(|e| {
                log::error!("Failed to await async systems, msg = {e:?}");
            })
            .unwrap();
        for async_system in async_systems {
            let _ = async_system.await.inspect_err(|e| {
                log::error!("Failed to await async system, msg = {e:?}");
            });
        }
    }
}
