use muleengine::{
    application_runner::SyncTaskSender, prelude::ResultInspector,
    sync::command_channel::CommandSender,
};

use super::{
    fps_camera_command::FpsCameraCommand, fps_camera_input_provider::FpsCameraInputSystem,
};

#[derive(Clone)]
pub struct FpsCameraClient {
    command_sender: CommandSender<FpsCameraCommand>,
    sync_tasks: SyncTaskSender,
}

impl FpsCameraClient {
    pub(super) fn new(
        command_sender: CommandSender<FpsCameraCommand>,
        sync_tasks: SyncTaskSender,
    ) -> Self {
        Self {
            command_sender,
            sync_tasks,
        }
    }

    pub fn stop(&self) {
        let _ = self
            .command_sender
            .send(FpsCameraCommand::Stop)
            .inspect_err(|e| log::error!("{e:?}"));
    }

    pub fn start(&self) {
        let _ = self
            .command_sender
            .send(FpsCameraCommand::Start)
            .inspect_err(|e| log::error!("{e:?}"));
    }

    pub fn remove_later(&self) {
        self.sync_tasks.add_task(|app_context| {
            app_context
                .system_container_mut()
                .remove::<FpsCameraInputSystem>();
            app_context
                .service_container_ref()
                .remove::<FpsCameraClient>();
        });
    }
}
