use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::{Command, CommandSender},
};

pub trait RendererTransform: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct TransformHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransformHandler(pub(crate) Arc<TransformHandlerDestructor>);

impl TransformHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
        Self(Arc::new(TransformHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for TransformHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TransformHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for TransformHandlerDestructor {}

impl PartialEq for TransformHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for TransformHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for TransformHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for TransformHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseTransform {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release transform, msg = {e}"));
    }
}
