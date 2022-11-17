use std::{
    any::{Any, TypeId},
    sync::Arc,
};

pub trait DowncastArc {
    fn downcast_arc<CastType: 'static>(&self) -> Option<Arc<CastType>>;
}

impl<T> DowncastArc for Arc<T>
where
    T: ?Sized + Any + 'static,
{
    fn downcast_arc<CastType: 'static>(&self) -> Option<Arc<CastType>> {
        let arc_clone = self.clone();

        if (*arc_clone).type_id() == TypeId::of::<CastType>() {
            let ptr = Arc::into_raw(arc_clone).cast::<CastType>();

            Some(unsafe { Arc::from_raw(ptr) })
        } else {
            None
        }
    }
}
