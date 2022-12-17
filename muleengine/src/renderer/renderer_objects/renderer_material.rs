use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::{Command, CommandSender},
};

pub trait RendererMaterial: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct MaterialHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaterialHandler(pub(crate) Arc<MaterialHandlerDestructor>);

impl MaterialHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
        Self(Arc::new(MaterialHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for MaterialHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MaterialHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for MaterialHandlerDestructor {}

impl PartialEq for MaterialHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for MaterialHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for MaterialHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for MaterialHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseMaterial {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release material, msg = {e}"));
    }
}
