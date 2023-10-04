use std::{panic, time::Duration};

use tokio::{
    sync::mpsc,
    time::{sleep_until, Instant},
};

use crate::{
    app_loop_state::AppLoopState, fps_counter::FpsCounter, prelude::ResultInspector,
    service_container::ServiceContainer, stopwatch::Stopwatch, system_container::SystemContainer,
};

pub type BoxedTask = Box<dyn FnOnce(&mut ApplicationContext) + Send>;

#[derive(Clone)]
pub struct ClosureTaskSender(mpsc::UnboundedSender<BoxedTask>);

impl ClosureTaskSender {
    pub fn add_task(&self, task: impl FnOnce(&mut ApplicationContext) + Send + 'static) {
        let _ = self
            .0
            .send(Box::new(task))
            .inspect_err(|_| log::error!("Could not add sync task, because receiver is destroyed"));
    }
}

pub struct ApplicationContext {
    system_container: SystemContainer,
    service_container: ServiceContainer,
    closure_tasks: ClosureTaskSender,
}

impl ApplicationContext {
    pub fn new(sync_task_sender: mpsc::UnboundedSender<BoxedTask>) -> Self {
        Self {
            system_container: SystemContainer::new(),
            service_container: ServiceContainer::new(),
            closure_tasks: ClosureTaskSender(sync_task_sender),
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

    pub fn closure_tasks_ref(&mut self) -> &ClosureTaskSender {
        &self.closure_tasks
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
    let (sync_task_sender, mut sync_task_receiver) = mpsc::unbounded_channel();
    let mut app_context = ApplicationContext::new(sync_task_sender);

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
        while let Ok(task) = sync_task_receiver.try_recv() {
            task(&mut app_context);
        }

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
