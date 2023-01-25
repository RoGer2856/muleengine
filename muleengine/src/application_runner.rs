use std::{panic, time::Duration};

use tokio::time::{sleep_until, Instant};

use crate::{
    app_loop_state::AppLoopState, fps_counter::FpsCounter, prelude::ResultInspector,
    service_container::ServiceContainer, stopwatch::Stopwatch, system_container::SystemContainer,
};

pub struct ApplicationContext {
    system_container: SystemContainer,
    service_container: ServiceContainer,
}

impl Default for ApplicationContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationContext {
    pub fn new() -> Self {
        Self {
            system_container: SystemContainer::new(),
            service_container: ServiceContainer::new(),
        }
    }

    pub fn system_container_ref(&self) -> &SystemContainer {
        &self.system_container
    }

    pub fn system_container_mut(&mut self) -> &mut SystemContainer {
        &mut self.system_container
    }

    pub fn service_container_ref(&self) -> &ServiceContainer {
        &self.service_container
    }
}

pub trait Application: 'static {
    fn should_run(&self, app_context: &mut ApplicationContext) -> bool;
    fn tick(&mut self, delta_time_in_secs: f32, app_context: &mut ApplicationContext);
}

pub fn run<ApplicationType>(
    multi_thread_executor: bool,
    application_creator_cb: impl FnOnce(&mut ApplicationContext) -> ApplicationType,
) where
    ApplicationType: Application,
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

pub async fn async_run<ApplicationType>(
    application_creator_cb: impl FnOnce(&mut ApplicationContext) -> ApplicationType,
) where
    ApplicationType: Application,
{
    let mut app_context = ApplicationContext::new();

    let mut application = application_creator_cb(&mut app_context);

    let app_loop_state = AppLoopState::new();
    let app_loop_state_watcher = app_loop_state.watcher();

    let old_panic_hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        log::error!("{panic_info}");
        app_loop_state.stop_loop();
    }));

    let mut fps_counter_stopwatch = Stopwatch::start_new();
    let mut fps_counter = FpsCounter::new();

    let mut delta_time_stopwatch = Stopwatch::start_new();
    let mut delta_time_in_secs = 1.0 / 60.0;

    while application.should_run(&mut app_context) && app_loop_state_watcher.should_run() {
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

    let drop_task = async move {
        drop(application);
        drop(app_context);
    };

    let timeout = Duration::from_secs(10);
    let timestamp = Instant::now() + timeout;
    tokio::select! {
        _ = sleep_until(timestamp) => {
            log::error!("Could not shutdown gracefully in {:?}", timeout);
        }
        _ = drop_task => {
        }
    }

    panic::set_hook(old_panic_hook);
}
