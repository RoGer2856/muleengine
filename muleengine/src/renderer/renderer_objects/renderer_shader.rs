use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    messaging::command_channel::CommandSender,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::Command,
};

pub trait RendererShader: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct ShaderHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender<Command>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ShaderHandler(pub(crate) Arc<ShaderHandlerDestructor>);

impl ShaderHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender<Command>) -> Self {
        Self(Arc::new(ShaderHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for ShaderHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for ShaderHandlerDestructor {}

impl PartialEq for ShaderHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for ShaderHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for ShaderHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for ShaderHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseShader {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release shader, msg = {e:?}"));
    }
}
