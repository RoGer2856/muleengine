use method_taskifier::task_channel::TaskSender;
use muleengine::{application_runner::ClosureTaskSender, prelude::ResultInspector};

use super::{
    fps_camera_command::FpsCameraCommand, fps_camera_input_provider::FpsCameraInputSystem,
};

#[derive(Clone)]
pub struct FpsCameraClient {
    task_sender: TaskSender<FpsCameraCommand>,
    closure_task_sender: ClosureTaskSender,
}

impl FpsCameraClient {
    pub(super) fn new(
        task_sender: TaskSender<FpsCameraCommand>,
        closure_task_sender: ClosureTaskSender,
    ) -> Self {
        Self {
            task_sender,
            closure_task_sender,
        }
    }

    pub fn stop(&self) {
        let _ = self
            .task_sender
            .send(FpsCameraCommand::Stop)
            .inspect_err(|e| log::error!("{e:?}"));
    }

    pub fn start(&self) {
        let _ = self
            .task_sender
            .send(FpsCameraCommand::Start)
            .inspect_err(|e| log::error!("{e:?}"));
    }

    pub fn remove_later(&self) {
        self.closure_task_sender.add_task(|app_context| {
            app_context
                .system_container_mut()
                .remove::<FpsCameraInputSystem>();
            app_context
                .service_container_ref()
                .remove::<FpsCameraClient>();
        });
    }
}
