mod result_option_inspect;

use std::{
    any::{Any, TypeId},
    rc::Rc,
    sync::Arc,
};

use parking_lot::{Mutex, RwLock};

pub use result_option_inspect::*;

pub type RcMutex<T> = Rc<Mutex<T>>;
pub type RcRwLock<T> = Rc<RwLock<T>>;

pub type ArcMutex<T> = Arc<Mutex<T>>;
pub type ArcRwLock<T> = Arc<RwLock<T>>;

pub fn rc_mutex_new<T>(object: T) -> RcMutex<T> {
    Rc::new(Mutex::new(object))
}

pub fn rc_rw_lock_new<T>(object: T) -> RcRwLock<T> {
    Rc::new(RwLock::new(object))
}

pub fn arc_mutex_new<T>(object: T) -> ArcMutex<T> {
    Arc::new(Mutex::new(object))
}

pub fn arc_rw_lock_new<T>(object: T) -> ArcRwLock<T> {
    Arc::new(RwLock::new(object))
}

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

pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
where
    T: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
