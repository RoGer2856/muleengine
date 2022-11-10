use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Mat4, Transform, Vec3};

use super::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex},
    drawable_object::DrawableObject,
};

struct Object {
    drawable: Arc<RwLock<dyn DrawableObject>>,
    transform: Mat4<f32>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct DrawableObjectStorageIndex(ObjectPoolIndex);

impl DrawableObjectStorageIndex {
    pub fn invalid() -> Self {
        Self(ObjectPoolIndex::invalid())
    }

    pub fn invalidate(&mut self) -> Self {
        let mut id = Self::invalid();
        std::mem::swap(&mut id, self);

        id
    }
}

pub struct DrawableObjectStorage {
    objects: ObjectPool<Object>,
}

impl DrawableObjectStorage {
    pub fn new() -> Self {
        Self {
            objects: ObjectPool::new(),
        }
    }

    pub fn add_drawable_object(
        &mut self,
        drawable_object: Arc<RwLock<dyn DrawableObject>>,
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
