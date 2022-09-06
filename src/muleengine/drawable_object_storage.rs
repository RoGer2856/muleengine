use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec3};

use crate::muleengine::object_pool::ObjectPool;

use super::{drawable_object::DrawableObject, object_pool::ObjectPoolIndex};

struct Object<DrawableObjectType: DrawableObject> {
    drawable: Arc<RwLock<DrawableObjectType>>,
    transform: Mat4<f32>,
}

pub struct DrawableObjectStorageIndex(ObjectPoolIndex);

pub struct DrawableObjectStorage<DrawableObjectType>
where
    DrawableObjectType: DrawableObject,
{
    objects: ObjectPool<Object<DrawableObjectType>>,
}

impl<DrawableObjectType> DrawableObjectStorage<DrawableObjectType>
where
    DrawableObjectType: DrawableObject,
{
    pub fn new() -> Self {
        Self {
            objects: ObjectPool::new(),
        }
    }

    pub fn add_drawable_object(
        &mut self,
        drawable_object: Arc<RwLock<DrawableObjectType>>,
        transform: Transform<f32, f32, f32>,
    ) -> DrawableObjectStorageIndex {
        DrawableObjectStorageIndex(self.objects.create_object(Object {
            drawable: drawable_object,
            transform: transform.into(),
        }))
    }

    pub fn render_all(
        &mut self,
        eye_position: &Vec3<f32>,
        projection_matrix: &Mat4<f32>,
        view_matrix: &Mat4<f32>,
    ) {
        for object in self.objects.iter_mut() {
            object.drawable.read().render(
                eye_position,
                projection_matrix,
                view_matrix,
                &object.transform,
            );
        }
    }
}
