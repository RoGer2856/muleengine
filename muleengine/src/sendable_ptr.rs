use std::{fmt::Display, marker::PhantomData};

pub struct SendablePtr<PtrType: ?Sized> {
    ptr: usize,
    _marker: PhantomData<PtrType>,
}

unsafe impl<PtrType: ?Sized> Send for SendablePtr<PtrType> {}
unsafe impl<PtrType: ?Sized> Sync for SendablePtr<PtrType> {}

impl<PtrType: ?Sized> Copy for SendablePtr<PtrType> {}

impl<PtrType: ?Sized> SendablePtr<PtrType> {
    pub fn new(ptr: *const PtrType) -> Self {
        Self {
            ptr: ptr as *const () as usize,
            _marker: PhantomData::default(),
        }
    }

    pub fn as_ptr(&self) -> *const () {
        self.ptr as *const ()
    }
}

impl<PtrType: ?Sized> Display for SendablePtr<PtrType> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SendablePtr")
            .field("ptr", &self.ptr)
            .finish()
    }
}

impl<PtrType: ?Sized> Clone for SendablePtr<PtrType> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr,
            _marker: self._marker,
        }
    }
}

impl<PtrType: ?Sized> Eq for SendablePtr<PtrType> {}

impl<PtrType: ?Sized> PartialEq for SendablePtr<PtrType> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<PtrType: ?Sized> Ord for SendablePtr<PtrType> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.ptr.cmp(&other.ptr)
    }
}

impl<PtrType: ?Sized> PartialOrd for SendablePtr<PtrType> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
