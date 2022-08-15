use vek::{Mat4, Transform};

use crate::muleengine::object_pool::ObjectPool;

use super::drawable_object::DrawableObject;

struct Object {
    drawable: Box<dyn DrawableObject>,
    transform: Mat4<f32>,
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
        drawable_object: Box<dyn DrawableObject>,
        transform: Transform<f32, f32, f32>,
    ) {
        self.objects.create_object(Object {
            drawable: drawable_object,
            transform: transform.into(),
        });
    }

    pub fn render_all(&mut self, projection_matrix: &Mat4<f32>, view_matrix: &Mat4<f32>) {
        for object in self.objects.iter_mut() {
            object
                .drawable
                .render(projection_matrix, view_matrix, &object.transform);
        }
    }
}
