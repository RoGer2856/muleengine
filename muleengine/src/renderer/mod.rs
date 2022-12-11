use std::{cmp::Ordering, fmt::Debug, sync::Arc};

use crate::result_option_inspect::ResultInspector;

use self::renderer_command::{Command, CommandSender};

use super::{containers::object_pool::ObjectPoolIndex, prelude::AsAny};

#[cfg(test)]
mod tests;

pub mod renderer_client;
mod renderer_command;
pub mod renderer_impl;
pub mod renderer_system;

pub trait RendererTransform: AsAny + Sync + Send + 'static {}
pub trait RendererMaterial: AsAny + Sync + Send + 'static {}
pub trait RendererShader: AsAny + Sync + Send + 'static {}
pub trait RendererMesh: AsAny + Sync + Send + 'static {}
pub trait RendererObject: AsAny + 'static {}

#[derive(Clone)]
struct TransformHandlerDestructor {
    object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TransformHandler(Arc<TransformHandlerDestructor>);

#[derive(Clone)]
struct MaterialHandlerDestructor {
    object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MaterialHandler(Arc<MaterialHandlerDestructor>);

#[derive(Clone)]
struct ShaderHandlerDestructor {
    object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ShaderHandler(Arc<ShaderHandlerDestructor>);

#[derive(Clone)]
struct MeshHandlerDestructor {
    object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeshHandler(Arc<MeshHandlerDestructor>);

#[derive(Clone)]
struct RendererObjectHandlerDestructor {
    object_pool_index: ObjectPoolIndex,
    command_sender: CommandSender,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RendererObjectHandler(Arc<RendererObjectHandlerDestructor>);

#[derive(Debug)]
pub enum RendererError {
    InvalidRendererTransformHandler(TransformHandler),
    InvalidRendererMaterialHandler(MaterialHandler),
    InvalidRendererShaderHandler(ShaderHandler),
    InvalidRendererMeshHandler(MeshHandler),
    InvalidRendererObjectHandler(RendererObjectHandler),
    RendererImplError(String),
}

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
            .inspect_err(|e| log::error!("Release transform, error = {e}"));
    }
}

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
            .inspect_err(|e| log::error!("Release material, error = {e}"));
    }
}

impl ShaderHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
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
            .inspect_err(|e| log::error!("Release shader, error = {e}"));
    }
}

impl MeshHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
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
            .inspect_err(|e| log::error!("Release mesh, error = {e}"));
    }
}

impl RendererObjectHandler {
    pub fn new(object_pool_index: ObjectPoolIndex, command_sender: CommandSender) -> Self {
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
            .inspect_err(|e| log::error!("Release renderer object, error = {e}"));
    }
}
