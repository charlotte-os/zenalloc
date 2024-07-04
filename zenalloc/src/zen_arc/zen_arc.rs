use core::sync::atomic::{AtomicUsize, Ordering};
use core::ptr::NonNull;
use core::ops::Deref;
use core::alloc::{Layout, AllocError};
use core::ptr;
use crate::System;
use crate::alloc_trait::Allocator;

pub struct ZenArc<T> {
    ptr: NonNull<ZenArcBox<T>>,
}

struct ZenArcBox<T> {
    value: T,
    ref_count: AtomicUsize,
}

impl<T> ZenArc<T> {
    pub fn new(value: T) -> Result<Self, AllocError> {
        let layout = Layout::new::<ZenArcBox<T>>();
        let ptr = System.allocate(layout)?;
        unsafe {
            let ptr = ptr.as_ptr() as *mut ZenArcBox<T>;
            ptr::write(ptr, ZenArcBox {
                value,
                ref_count: AtomicUsize::new(1),
            });
            Ok(ZenArc {
                ptr: NonNull::new_unchecked(ptr),
            })
        }
    }

    pub fn strong_count(this: &Self) -> usize {
        unsafe { this.ptr.as_ref().ref_count.load(Ordering::SeqCst) }
    }

    pub fn clone(this: &Self) -> Self {
        let count = ZenArc::strong_count(this);
        unsafe {
            this.ptr.as_ref().ref_count.fetch_add(1, Ordering::SeqCst);
        }
        ZenArc {
            ptr: this.ptr,
        }
    }
}

impl<T> Deref for ZenArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &self.ptr.as_ref().value }
    }
}

impl<T> Drop for ZenArc<T> {
    fn drop(&mut self) {
        let count = ZenArc::strong_count(self);
        if count == 1 {
            let layout = Layout::new::<ZenArcBox<T>>();
            unsafe {
                ptr::drop_in_place(self.ptr.as_ptr());
                System.deallocate(self.ptr.cast(), layout);
            }
        } else {
            unsafe {
                self.ptr.as_ref().ref_count.fetch_sub(1, Ordering::SeqCst);
            }
        }
    }
}
