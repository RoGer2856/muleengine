use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::{Command, CommandSender},
};

pub trait RendererCamera: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct CameraHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CameraHandler(pub(crate) Arc<CameraHandlerDestructor>);

impl CameraHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
        Self(Arc::new(CameraHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for CameraHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CameraHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for CameraHandlerDestructor {}

impl PartialEq for CameraHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for CameraHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for CameraHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for CameraHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseCamera {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release camera, msg = {e}"));
    }
}
