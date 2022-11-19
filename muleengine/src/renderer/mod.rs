use super::{containers::object_pool::ObjectPoolIndex, prelude::AsAny};

pub mod renderer_client;
mod renderer_command;
pub mod renderer_impl;
pub mod renderer_system;

pub trait Shader: AsAny + 'static {}
pub trait DrawableMesh: AsAny + 'static {}
pub trait DrawableObject: AsAny + 'static {}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ShaderId(ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DrawableMeshId(ObjectPoolIndex);
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DrawableObjectId(ObjectPoolIndex);

#[derive(Debug)]
pub enum RendererError {
    InvalidShaderId(ShaderId),
    InvalidDrawableMeshId(DrawableMeshId),
    InvalidDrawableObjectId(DrawableObjectId),
    RendererImplError(String),
}
