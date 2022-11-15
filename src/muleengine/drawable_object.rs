use std::any::Any;

pub trait DrawableObjectAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub trait DrawableObject: DrawableObjectAny + 'static {}

impl<T> DrawableObjectAny for T
where
    T: DrawableObject,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
