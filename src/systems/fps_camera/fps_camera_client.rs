use muleengine::{prelude::ResultInspector, sync::command_channel::CommandSender};

use super::fps_camera_command::FpsCameraCommand;

pub struct FpsCameraClient {
    command_sender: CommandSender<FpsCameraCommand>,
}

impl FpsCameraClient {
    pub(super) fn new(command_sender: CommandSender<FpsCameraCommand>) -> Self {
        Self { command_sender }
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
}
