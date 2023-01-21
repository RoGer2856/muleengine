use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::Command,
    sync::command_channel::CommandSender,
};

pub trait RendererObject: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct RendererObjectHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender<Command>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererObjectHandler(pub(crate) Arc<RendererObjectHandlerDestructor>);

impl RendererObjectHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender<Command>) -> Self {
        Self(Arc::new(RendererObjectHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for RendererObjectHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererObjectHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for RendererObjectHandlerDestructor {}

impl PartialEq for RendererObjectHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for RendererObjectHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for RendererObjectHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for RendererObjectHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseRendererObject {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release renderer object, msg = {e:?}"));
    }
}
