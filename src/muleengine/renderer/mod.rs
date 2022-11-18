use super::{containers::object_pool::ObjectPoolIndex, prelude::AsAny};

pub mod renderer_client;
mod renderer_command;
pub mod renderer_impl;
pub mod renderer_system;

pub trait DrawableMesh: AsAny + 'static {}
pub trait DrawableObject: AsAny + 'static {}

#[derive(Debug)]
pub struct DrawableMeshId(ObjectPoolIndex);
#[derive(Debug)]
pub struct DrawableObjectId(ObjectPoolIndex);

#[derive(Debug)]
pub enum RendererError {
    InvalidDrawableMeshId(DrawableMeshId),
    InvalidDrawableObjectId(DrawableObjectId),
    RendererImplError(String),
}
