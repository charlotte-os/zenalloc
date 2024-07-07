use core::cell::Cell;
use core::ptr::NonNull;
use core::ops::Deref;
use core::alloc::{Layout, AllocError};
use core::ptr;
use crate::System;
use crate::alloc_trait::Allocator;

pub struct ZenRc<T> {
    ptr: NonNull<ZenRcBox<T>>,
}

struct ZenRcBox<T> {
    value: T,
    ref_count: Cell<usize>,
}

impl<T> ZenRc<T> {
    pub fn new(value: T) -> Result<Self, AllocError> {
        let layout = Layout::new::<ZenRcBox<T>>();
        let ptr = System.allocate(layout)?;
        unsafe {
            let ptr = ptr.as_ptr() as *mut ZenRcBox<T>;
            ptr::write(ptr, ZenRcBox {
                value,
                ref_count: Cell::new(1),
            });
            Ok(ZenRc {
                ptr: NonNull::new_unchecked(ptr),
            })
        }
    }

    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.ptr.as_ref().ref_count.get() }
    }

    pub fn clone(this: &Self) -> Self {
        let count = ZenRc::strong_count(this);
        unsafe {
            this.ptr.as_ref().ref_count.set(count + 1);
        }
        ZenRc {
            ptr: this.ptr,
        }
    }
}

impl<T> Deref for ZenRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T> Drop for ZenRc<T> {
    fn drop(&mut self) {
        let count = ZenRc::strong_count(self);
        if count == 1 {
            let layout = Layout::new::<ZenRcBox<T>>();
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr());
                System.deallocate(self.ptr.cast(), layout);
            }
        } else {
            unsafe {
                self.ptr.as_ref().ref_count.set(count - 1);
            }
        }
    }
}

unsafe impl<T: Send + Sync> Send for ZenRc<T> {}
unsafe impl<T: Sync> Sync for ZenRc<T> {}