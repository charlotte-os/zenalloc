use core::alloc::Layout;
use core::ptr::{self, NonNull};
use crate::alloc_trait::Allocator;
use crate::System;

pub struct Box<T> {
    ptr: NonNull<T>,
}

impl<T> Box<T> {
    // Allocate memory for a single value of type T
    pub fn new(value: T) -> Result<Self, AllocError> {
        // Create a layout for a single value of type T
        let layout = Layout::new::<T>();
        // Allocate memory using the custom allocator
        let ptr = System.allocate(layout)?;
        unsafe {
            // Write the value to the allocated memory
            ptr::write(ptr.as_ptr() as *mut T, value);
        }
        Ok(Box { ptr: ptr.cast() })
    }

    // Get a reference to the contained value
    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    // Get a mutable reference to the contained value
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        // Deallocate the memory when the box is dropped
        let layout = Layout::new::<T>();
        unsafe {
            // Drop the value
            ptr::drop_in_place(self.ptr.as_ptr());
            // Deallocate the memory
            System.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T> core::ops::Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.as_ref()
    }
}

impl<T> core::ops::DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.as_mut()
    }
}
