use core::alloc::{AllocError, Layout};
use core::ptr::{self, NonNull};
use core::sync::atomic::{AtomicUsize, Ordering};
use crate::alloc_trait::Allocator;
use crate::System;

pub struct ZenCow<T: Clone> {
    ptr: NonNull<T>,
    ref_count: NonNull<AtomicUsize>,
}

impl<T: Clone> ZenCow<T> {
    pub fn new(value: T) -> Result<Self, AllocError> {
        let layout = Layout::new::<T>();
        let ptr = System.allocate(layout)?;
        unsafe {
            ptr::write(ptr.as_ptr() as *mut T, value);
        }

        let count_layout = Layout::new::<AtomicUsize>();
        let ref_count = System.allocate(count_layout)?;
        unsafe {
            ptr::write(ref_count.as_ptr() as *mut AtomicUsize, AtomicUsize::new(1));
        }

        Ok(ZenCow {
            ptr: ptr.cast(),
            ref_count: ref_count.cast(),
        })
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn get_mut(&mut self) -> &mut T {
        if self.ref_count().load(Ordering::SeqCst) != 1 {
            // Clone the data
            let layout = Layout::new::<T>();
            let new_ptr = System.allocate(layout).unwrap();
            unsafe {
                ptr::write(new_ptr.as_ptr() as *mut T, self.ptr.as_ref().clone());
            }
            self.ptr = new_ptr.cast();

            // Update the reference count
            self.ref_count().fetch_sub(1, Ordering::SeqCst);
            let count_layout = Layout::new::<AtomicUsize>();
            let new_ref_count = System.allocate(count_layout).unwrap();
            unsafe {
                ptr::write(new_ref_count.as_ptr() as *mut AtomicUsize, AtomicUsize::new(1));
            }
            self.ref_count = new_ref_count.cast();
        }
        unsafe { self.ptr.as_mut() }
    }

    fn ref_count(&self) -> &AtomicUsize {
        unsafe { self.ref_count.as_ref() }
    }
}

impl<T: Clone> Clone for ZenCow<T> {
    fn clone(&self) -> Self {
        self.ref_count().fetch_add(1, Ordering::SeqCst);
        ZenCow {
            ptr: self.ptr,
            ref_count: self.ref_count,
        }
    }
}

impl<T: Clone> Drop for ZenCow<T> {
    fn drop(&mut self) {
        if self.ref_count().fetch_sub(1, Ordering::SeqCst) == 1 {
            let layout = Layout::new::<T>();
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr());
                System.deallocate(self.ptr.cast(), layout);
            }

            let count_layout = Layout::new::<AtomicUsize>();
            unsafe {
                System.deallocate(self.ref_count.cast(), count_layout);
            }
        }
    }
}
