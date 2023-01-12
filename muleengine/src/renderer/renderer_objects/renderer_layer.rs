use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    messaging::command_channel::CommandSender,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::Command,
};

pub trait RendererLayer: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct RendererLayerHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender<Command>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererLayerHandler(pub(crate) Arc<RendererLayerHandlerDestructor>);

impl RendererLayerHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender<Command>) -> Self {
        Self(Arc::new(RendererLayerHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for RendererLayerHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererLayerHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for RendererLayerHandlerDestructor {}

impl PartialEq for RendererLayerHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for RendererLayerHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for RendererLayerHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for RendererLayerHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseRendererLayer {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release renderer layer, msg = {e:?}"));
    }
}
