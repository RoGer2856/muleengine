use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::{
    containers::object_pool::ObjectPoolIndex,
    prelude::{AsAny, ResultInspector},
    renderer::renderer_command::Command,
    sync::command_channel::CommandSender,
};

pub trait RendererMesh: AsAny + Sync + Send + 'static {}

#[derive(Clone)]
pub(crate) struct MeshHandlerDestructor {
    pub(crate) object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender<Command>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeshHandler(pub(crate) Arc<MeshHandlerDestructor>);

impl MeshHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender<Command>) -> Self {
        Self(Arc::new(MeshHandlerDestructor {
            object_pool_index,
            command_sender,
        }))
    }
}

impl Debug for MeshHandlerDestructor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MeshHandlerDestructor")
            .field("object_pool_index", &self.object_pool_index)
            .finish()
    }
}

impl Eq for MeshHandlerDestructor {}

impl PartialEq for MeshHandlerDestructor {
    fn eq(&self, other: &Self) -> bool {
        self.object_pool_index == other.object_pool_index
    }
}

impl Ord for MeshHandlerDestructor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.object_pool_index.cmp(&other.object_pool_index)
    }
}

impl PartialOrd for MeshHandlerDestructor {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Drop for MeshHandlerDestructor {
    fn drop(&mut self) {
        let _ = self
            .command_sender
            .send(Command::ReleaseMesh {
                object_pool_index: self.object_pool_index,
            })
            .inspect_err(|e| log::error!("Release mesh, msg = {e:?}"));
    }
}
