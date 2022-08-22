use std::{mem::swap, sync::Arc};

use vek::{Mat4, Transform};

use crate::muleengine::object_pool::ObjectPool;

use super::{drawable_object::DrawableObject, object_pool::ObjectPoolIndex};

struct Object {
    drawable: Arc<dyn DrawableObject>,
    transform: Mat4<f32>,
}

pub struct DrawableObjectStorageIndex(ObjectPoolIndex);

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
        drawable_object: Arc<dyn DrawableObject>,
        transform: Transform<f32, f32, f32>,
    ) -> DrawableObjectStorageIndex {
        DrawableObjectStorageIndex(self.objects.create_object(Object {
            drawable: drawable_object,
            transform: transform.into(),
        }))
    }

    pub fn replace_drawable_object(
        &mut self,
        mut new_drawable_object: Arc<dyn DrawableObject>,
        index: DrawableObjectStorageIndex,
    ) -> Option<Arc<dyn DrawableObject>> {
        match self.objects.get_mut(index.0) {
            Some(object) => {
                swap(&mut object.drawable, &mut new_drawable_object);
                Some(new_drawable_object)
            }
            None => None,
        }
    }

    pub fn render_all(&mut self, projection_matrix: &Mat4<f32>, view_matrix: &Mat4<f32>) {
        for object in self.objects.iter_mut() {
            object
                .drawable
                .render(projection_matrix, view_matrix, &object.transform);
        }
    }
}
