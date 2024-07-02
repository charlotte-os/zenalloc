use core::ptr::NonNull;
use core::alloc::Layout;
use core::mem;
use core::ops::{Deref, DerefMut};
use crate::alloc_trait::Allocator;

pub struct Box<T, A: Allocator> {
    ptr: NonNull<T>,
    allocator: A,
    layout: Layout,
}

impl<T, A: Allocator> Box<T, A> {
    pub fn new_in(value: T, allocator: A) -> Result<Self, AllocError> {
        // Determine the layout for type T
        let layout = Layout::new::<T>();
        
        // Allocate memory for T using the custom allocator
        let mem = allocator.allocate(layout)?.as_ptr() as *mut T;

        // SAFETY: The allocation was successful, so we can assume `mem` is a valid pointer.
        unsafe {
            mem::write(mem, value);
        }

        Ok(Box {
            ptr: NonNull::new(mem).ok_or(AllocError)?,
            allocator,
            layout,
        })
    }
}

impl<T, A: Allocator> Deref for Box<T, A> {
    type Target = T;

    fn deref(&self) -> &T {
        // SAFETY: The pointer is guaranteed to be valid as long as `Box` is valid.
        unsafe { self.ptr.as_ref() }
    }
}

impl<T, A: Allocator> DerefMut for Box<T, A> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: The pointer is guaranteed to be valid as long as `Box` is valid.
        unsafe { self.ptr.as_mut() }
    }
}

impl<T, A: Allocator> Drop for Box<T, A> {
    fn drop(&mut self) {
        // SAFETY: The pointer is guaranteed to be valid as long as `Box` is valid.
        unsafe {
            // Drop the value
            ptr::drop_in_place(self.ptr.as_ptr());

            // Deallocate the memory
            self.allocator.deallocate(self.ptr.cast(), self.layout);
        }
    }
}
