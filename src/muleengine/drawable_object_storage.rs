use std::sync::Arc;

use parking_lot::RwLock;

use super::{
    containers::object_pool::{ObjectPool, ObjectPoolIndex, ObjectPoolIter},
    drawable_object::DrawableObject,
};

struct Object {
    drawable: Arc<RwLock<dyn DrawableObject>>,
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
    type Item = &'a Arc<RwLock<dyn DrawableObject>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iterator.next().map(|object| &object.drawable)
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
    ) -> DrawableObjectStorageIndex {
        DrawableObjectStorageIndex(self.objects.create_object(Object {
            drawable: drawable_object,
        }))
    }

    pub fn iter(&self) -> DrawableObjectStorageIter {
        DrawableObjectStorageIter {
            inner_iterator: self.objects.iter(),
        }
    }

    pub fn get(&self, id: DrawableObjectStorageIndex) -> Option<&Arc<RwLock<dyn DrawableObject>>> {
        self.objects.get_ref(id.0).map(|object| &object.drawable)
    }
}
