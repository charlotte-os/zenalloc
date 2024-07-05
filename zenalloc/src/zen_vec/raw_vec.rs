use core::alloc::{Layout, AllocError};
use core::ptr::NonNull;
use crate::alloc_trait::Allocator;
use crate::System;

pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    pub fn new() -> Self {
        RawVec {
            ptr: NonNull::dangling(), // Initializes the pointer to a non-null dangling pointer
            cap: 0,                   // Initializes the capacity to 0
        }
    }

    // Creates a new `RawVec` with the specified capacity.
    pub fn with_capacity(capacity: usize) -> Result<Self, AllocError> {
        // If the requested capacity is 0, return an empty `RawVec`.
        if capacity == 0 {
            return Ok(Self::new());
        }

        // Calculate the layout for the requested capacity.
        let layout = Layout::array::<T>(capacity).map_err(|_| AllocError)?;

        // Allocate the memory using the custom allocator.
        let ptr = System.allocate(layout)?;

        Ok(RawVec {
            ptr: ptr.cast(),
            cap: capacity,
        })
    }

    // Grows the capacity of the `RawVec` by doubling it.
    pub fn grow(&mut self) -> Result<(), AllocError> {
        // Calculate the new capacity (double the current capacity, or 1 if the current capacity is 0).
        let new_capacity = if self.cap == 0 { 1 } else { 2 * self.cap };

        // Calculate the layouts for the old and new capacities.
        let old_layout = Layout::array::<T>(self.cap).map_err(|_| AllocError)?;
        let new_layout = Layout::array::<T>(new_capacity).map_err(|_| AllocError)?;

        // Reallocate the memory to the new capacity using the custom allocator.
        let new_ptr = unsafe { System.grow(self.ptr.cast(), old_layout, new_layout)? };

        // Update the pointer and capacity to the new values.
        self.ptr = new_ptr.cast();
        self.cap = new_capacity;

        Ok(())
    }

    // Returns the current capacity of the `RawVec`.
    pub fn capacity(&self) -> usize {
        self.cap
    }

    // Returns the pointer to the allocated memory.
    pub fn ptr(&self) -> NonNull<T> {
        self.ptr
    }
}

// Implements the `Drop` trait for `RawVec` to ensure memory is properly deallocated.
impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        // If the capacity is not 0, deallocate the memory.
        if self.cap != 0 {
            let layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { System.deallocate(self.ptr.cast(), layout) };
        }
    }
}
