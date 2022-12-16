use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::{Command, CommandSender},
};

pub trait RendererGroup: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct RendererGroupHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererGroupHandler(pub(crate) Arc<RendererGroupHandlerDestructor>);

impl RendererGroupHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
        Self(Arc::new(RendererGroupHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for RendererGroupHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererGroupHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for RendererGroupHandlerDestructor {}

impl PartialEq for RendererGroupHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for RendererGroupHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for RendererGroupHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for RendererGroupHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseRendererGroup {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release renderer group, error = {e}"));
    }
}
