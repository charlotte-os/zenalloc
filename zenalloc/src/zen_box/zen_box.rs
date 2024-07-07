use core::alloc::{AllocError, Layout};
use core::ptr::{self, NonNull};
use crate::alloc_trait::Allocator;
use crate::System;

pub struct ZenBox<T> {
    ptr: NonNull<T>,
}

impl<T> ZenBox<T> {
    pub fn new(value: T) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = System.allocate(layout)?;
        unsafe {
            ptr::write(ptr.as_ptr() as *mut T, value);
        }
        Ok(ZenBox { ptr: ptr.cast() })
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for ZenBox<T> {
    fn drop(&mut self) {
        let layout = Layout::new::<T>();
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            System.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T> core::ops::Deref for ZenBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> core::ops::DerefMut for ZenBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}

unsafe impl<T: Send> Send for ZenBox<T> {}
unsafe impl<T: Sync> Sync for ZenBox<T> {}
