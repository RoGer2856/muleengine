use std::future::Future;

use tokio::task::JoinHandle;

use crate::prelude::ResultInspector;

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
        if let Ok(async_systems) = self.async_systems.await.inspect_err(|e| {
            log::error!("awaiting async systems, msg = {e:?}");
        }) {
            for async_system in async_systems {
                let _ = async_system.await.inspect_err(|e| {
                    log::error!("awaiting async system, msg = {e:?}");
                });
            }
        }
    }
}
