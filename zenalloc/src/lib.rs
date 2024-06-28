#![no_std]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

extern crate alloc;

use core::alloc::{GlobalAlloc, Layout, AllocError};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, ptr};

mod alloc_trait;

pub struct System;

unsafe impl GlobalAlloc for System {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        // Dummy implementation
        ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // Dummy implementation
    }
}

impl System {
    #[inline]
    fn alloc_impl(&self, layout: Layout, zeroed: bool) -> Result<NonNull<[u8]>, AllocError> {
        match layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(layout.dangling(), 0)),
            _ => {
                let raw_ptr: *mut u8 = unsafe {
                    if zeroed {
                        System.alloc(layout)
                    } else {
                        System.alloc(layout)
                    }
                };

                let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
                Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
            }
        }
    }

    #[inline]
    unsafe fn grow_impl(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
        zeroed: bool,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        match old_layout.size() {
            0 => self.alloc_impl(new_layout, zeroed),
            old_size if old_layout.align() == new_layout.align() => {
                let new_size = new_layout.size();
                let raw_ptr = System.alloc(new_layout);
                let new_ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;

                // core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size);
                if zeroed {
                    new_ptr.as_ptr().add(old_size).write_bytes(0, new_size - old_size);
                }
                Ok(NonNull::slice_from_raw_parts(new_ptr, new_size))
            }
            old_size => {
                let new_ptr = self.alloc_impl(new_layout, zeroed)?;
                // core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size);
                Ok(new_ptr)
            }
        }
    }
}

impl alloc_trait::Allocator for System {
    #[inline]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, false)
    }

    #[inline]
    fn allocate_zeroed(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        self.alloc_impl(layout, true)
    }

    #[inline]
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        System.dealloc(ptr.as_ptr(), layout);
    }

    #[inline]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.grow_impl(ptr, old_layout, new_layout, false)
    }

    #[inline]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        self.grow_impl(ptr, old_layout, new_layout, true)
    }

    #[inline]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        match new_layout.size() {
            0 => Ok(NonNull::slice_from_raw_parts(new_layout.dangling(), 0)),
            new_size if old_layout.align() == new_layout.align() => {
                Ok(NonNull::slice_from_raw_parts(ptr, new_size))
            }
            new_size => {
                let new_ptr = self.allocate(new_layout)?;
                // core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_size);
                Ok(new_ptr)
            }
        }
    }
}

static HOOK: AtomicPtr<()> = AtomicPtr::new(ptr::null_mut());

pub fn set_alloc_error_hook(hook: fn(Layout)) {
    HOOK.store(hook as *mut (), Ordering::Release);
}

pub fn take_alloc_error_hook() -> fn(Layout) {
    let hook = HOOK.swap(ptr::null_mut(), Ordering::Acquire);
    if hook.is_null() {
        default_alloc_error_hook
    } else {
        unsafe { mem::transmute(hook) }
    }
}

fn default_alloc_error_hook(_layout: Layout) {
    // Using print macro here instead of eprintln because no_std
    // Printing might not be available in all no_std environments
}

#[cfg_attr(not(test), alloc_error_handler)]
fn rust_oom(layout: Layout) -> ! {
    let hook = HOOK.load(Ordering::Acquire);
    let hook: fn(Layout) =
        if hook.is_null() {
            default_alloc_error_hook
        } else {
            unsafe { mem::transmute(hook) }
        };
    hook(layout);
    loop {}
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use core::alloc::Layout;
    use crate::alloc_trait::Allocator;

    #[test]
    fn test_basic_allocation() {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let ptr = System.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());
        unsafe { System.deallocate(ptr.cast(), layout) };
    }

    #[test]
    fn test_allocation_zeroed() {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let ptr = System.allocate_zeroed(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());
        // let slice = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), 1024) };
        // assert!(slice.iter().all(|&byte| byte == 0));
        unsafe { System.deallocate(ptr.cast(), layout) };
    }

    #[test]
    fn test_reallocation() {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let ptr = System.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());

        let new_layout = Layout::from_size_align(2048, 8).unwrap();
        let new_ptr = unsafe { System.grow(ptr.cast(), layout, new_layout) }.unwrap();
        assert!(!new_ptr.as_ptr().is_null());

        unsafe { System.deallocate(new_ptr.cast(), new_layout) };
    }

    #[test]
    fn test_shrink_allocation() {
        let layout = Layout::from_size_align(2048, 8).unwrap();
        let ptr = System.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());

        let new_layout = Layout::from_size_align(1024, 8).unwrap();
        let new_ptr = unsafe { System.shrink(ptr.cast(), layout, new_layout) }.unwrap();
        assert!(!new_ptr.as_ptr().is_null());

        unsafe { System.deallocate(new_ptr.cast(), new_layout) };
    }

    #[test]
    fn test_grow_allocation() {
        let layout = Layout::from_size_align(1024, 8).unwrap();
        let ptr = System.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());

        let new_layout = Layout::from_size_align(2048, 8).unwrap();
        let new_ptr = unsafe { System.grow(ptr.cast(), layout, new_layout) }.unwrap();
        assert!(!new_ptr.as_ptr().is_null());

        unsafe { System.deallocate(new_ptr.cast(), new_layout) };
    }

    #[test]
    fn test_deallocation_of_zero_sized_layout() {
        let layout = Layout::from_size_align(0, 1).unwrap();
        let ptr = System.allocate(layout).unwrap();
        assert!(!ptr.as_ptr().is_null());
        unsafe { System.deallocate(ptr.cast(), layout) };
    }

    #[test]
    fn test_alloc_error_hook() {
        set_alloc_error_hook(|layout| {
            assert_eq!(layout.size(), 0);
            assert_eq!(layout.align(), 1);
        });

        let layout = Layout::from_size_align(0, 1).unwrap();
        let _ = System.allocate(layout);

        take_alloc_error_hook();
    }
}
