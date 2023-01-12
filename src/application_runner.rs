use std::time::Duration;

use muleengine::{
    fps_counter::FpsCounter, prelude::ResultInspector, service_container::ServiceContainer,
    stopwatch::Stopwatch, system_container::SystemContainer,
};
use tokio::time::{sleep_until, Instant};

use crate::async_systems_runner::AsyncSystemsRunner;

pub struct ApplicationContext {
    system_container: SystemContainer,
    service_container: ServiceContainer,
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self {
            system_container: SystemContainer::new(),
            service_container: ServiceContainer::new(),
        }
    }

    pub fn system_container(&mut self) -> &mut SystemContainer {
        &mut self.system_container
    }

    pub fn service_container(&mut self) -> &mut ServiceContainer {
        &mut self.service_container
    }
}

pub trait ApplicationCallbacks: 'static {
    fn async_systems(&mut self) -> AsyncSystemsRunner;
    fn should_run(&self, app_context: &mut ApplicationContext) -> bool;
    fn tick(&mut self, delta_time_in_secs: f32, app_context: &mut ApplicationContext);
}

pub fn run<ACType>(
    multi_thread_executor: bool,
    application_creator_cb: impl FnOnce(&mut ApplicationContext) -> ACType,
) where
    ACType: ApplicationCallbacks,
{
    let rt = if multi_thread_executor {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .inspect_err(|e| log::error!("Could not create tokio multi thread runtime, msg = {e}"))
            .unwrap()
    } else {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .inspect_err(|e| {
                log::error!("Could not create tokio current thread runtime, msg = {e}")
            })
            .unwrap()
    };

    rt.block_on(async_run(application_creator_cb));
}

pub async fn async_run<ACType>(
    application_creator_cb: impl FnOnce(&mut ApplicationContext) -> ACType,
) where
    ACType: ApplicationCallbacks,
{
    let mut app_context = ApplicationContext::new();

    let mut application = application_creator_cb(&mut app_context);

    let async_systems = application.async_systems();
    let async_systems = tokio::spawn(async move {
        async_systems.join().await;
    });

    let mut fps_counter_stopwatch = Stopwatch::start_new();
    let mut fps_counter = FpsCounter::new();

    let mut delta_time_stopwatch = Stopwatch::start_new();
    let mut delta_time_in_secs = 1.0 / 60.0;

    while application.should_run(&mut app_context) {
        app_context.system_container.tick(delta_time_in_secs);

        application.tick(delta_time_in_secs, &mut app_context);

        fps_counter.draw_happened();
        if fps_counter_stopwatch.elapsed() > Duration::from_millis(1000) {
            fps_counter_stopwatch.restart();
            fps_counter.restart();
        }

        tokio::task::yield_now().await;

        delta_time_in_secs = delta_time_stopwatch.restart().as_secs_f32();
    }

    drop(application);
    drop(app_context);

    let timeout = Duration::from_secs(10);
    let timestamp = Instant::now() + timeout;
    tokio::select! {
        _ = sleep_until(timestamp) => {
            log::error!("Could not shutdown gracefully in {:?}", timeout);
        }
        join_result = async_systems => {
            let _ = join_result.inspect_err(|e| log::error!("Joining async systems, msg = {e}"));
        }
    }
}
