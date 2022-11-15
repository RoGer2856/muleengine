use std::sync::Arc;

use parking_lot::RwLock;
use vek::{Mat4, Transform};

use super::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex, ObjectPoolIter},
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

pub struct DrawableObjectStorageIter<'a> {
    inner_iterator: ObjectPoolIter<'a, Object>,
}

impl<'a> Iterator for DrawableObjectStorageIter<'a> {
    type Item = (&'a Mat4<f32>, &'a Arc<RwLock<dyn DrawableObject>>);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iterator
            .next()
            .map(|object| (&object.transform, &object.drawable))
    }
}

pub struct DrawableObjectStorage {
    objects: ObjectPool<Object>,
}

impl Default for DrawableObjectStorage {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn iter(&self) -> DrawableObjectStorageIter {
        DrawableObjectStorageIter {
            inner_iterator: self.objects.iter(),
        }
    }
}
