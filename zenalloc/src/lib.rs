#![no_std]
#![feature(alloc_error_handler)]
#![feature(allocator_api)]
#![feature(alloc_layout_extra)]

extern crate alloc;
use alloc::alloc::{alloc, alloc_zeroed, dealloc, realloc};

use crate::alloc_trait::Allocator;
use core::alloc::{AllocError, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};
use core::{mem, ptr};

mod alloc_trait;
mod zen_arc;
mod zen_box;
mod zen_cow;
mod zen_rc;
mod zen_string;
mod zen_vec;

pub struct System;

impl System {
    #[inline]
    fn alloc_impl(&self, layout: Layout, zeroed: bool) -> Result<NonNull<[u8]>, AllocError> {
        if layout.size() == 0 {
            return Ok(NonNull::slice_from_raw_parts(NonNull::dangling(), 0));
        }

        let raw_ptr: *mut u8 = if zeroed {
            unsafe { alloc_zeroed(layout) }
        } else {
            unsafe { alloc(layout) }
        };

        if raw_ptr.is_null() {
            return Err(AllocError);
        }

        let ptr = NonNull::new(raw_ptr).ok_or(AllocError)?;
        Ok(NonNull::slice_from_raw_parts(ptr, layout.size()))
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

        if old_layout.size() == 0 {
            return self.alloc_impl(new_layout, zeroed);
        }

        let new_size = new_layout.size();
        let raw_ptr = realloc(ptr.as_ptr(), old_layout, new_size);

        if raw_ptr.is_null() {
            return Err(AllocError);
        }

        let new_ptr = NonNull::new_unchecked(raw_ptr);

        if zeroed && new_size > old_layout.size() {
            new_ptr
                .as_ptr()
                .add(old_layout.size())
                .write_bytes(0, new_size - old_layout.size());
        }

        Ok(NonNull::slice_from_raw_parts(new_ptr, new_size))
    }
}

impl Allocator for System {
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
        if layout.size() != 0 {
            dealloc(ptr.as_ptr(), layout);
        }
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
            0 => Ok(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
            new_size if old_layout.align() == new_layout.align() => {
                Ok(NonNull::slice_from_raw_parts(ptr, new_size))
            }
            new_size => {
                let new_ptr = self.allocate(new_layout)?;
                core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr() as *mut u8, new_size);
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
    let hook: fn(Layout) = if hook.is_null() {
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
    use zen_vec::zen_vec::VecError;

    use super::*;
    use crate::alloc_trait::Allocator;
    use crate::zen_arc::zen_arc::ZenArc;
    use crate::zen_box::zen_box::ZenBox;
    use crate::zen_cow::zen_cow::ZenCow;
    use crate::zen_rc::zen_rc::ZenRc;
    use crate::zen_string::zen_ascii_string::ZenAsciiString;
    use crate::zen_vec::raw_vec::RawVec;
    use crate::zen_vec::zen_vec::ZenVec;
    use core::alloc::Layout;

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
        let slice = unsafe { ptr.as_ref() };
        assert!(slice.iter().all(|&byte| byte == 0));
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

    #[test]
    fn test_raw_vec() {
        let mut vec = RawVec::<u8>::with_capacity(1024).unwrap();
        assert_eq!(vec.capacity(), 1024);

        vec.grow().unwrap();
        assert_eq!(vec.capacity(), 2048);
    }

    #[test]
    fn test_zen_vec_ops() {
        let mut vec: ZenVec<u8> = ZenVec::new();
        assert!(vec.is_empty(), "ZenVec is_empty() failed");

        assert!(vec.push(0) == Ok(()), "ZenVec push() return failed");
        assert!(vec == [0], "ZenVec push() failed");

        let value = vec.pop();
        assert!(value == Some(0), "ZenVec pop() failed");

        assert!(vec.push(0) == Ok(()), "ZenVec push() #1 return failed");
        assert!(vec.push(1) == Ok(()), "ZenVec push() #2 return failed");
        assert!(vec.push(2) == Ok(()), "ZenVec push() #3 return failed");
        assert!(vec == [0, 1, 2], "ZenVec multi-push failed");

        assert!(vec.remove(3) == Err(VecError::IndexOutOfBounds), "ZenVec index checking failed");
        
        assert!(vec.remove(1) == Ok(1), "ZenVec remove() return failed");
        assert!(vec == [0, 2], "ZenVec remove() failed");

        assert!(vec.insert(1, 1) == Ok(()), "ZenVec insert() return failed");
        assert!(vec == [0, 1, 2], "ZenVec insert() failed");
    }

    #[test]
    fn test_zen_vec_iter() {
        let mut vec: ZenVec<u8> = ZenVec::new();
        for i in 1..4 {
            vec.push(i).unwrap();
        }

        let mut iter = vec.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zen_vec_iter_mut() {
        let mut vec: ZenVec<u8> = ZenVec::new();
        for i in 1..4 {
            vec.push(i).unwrap();
        }

        let mut iter = vec.iter_mut();
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_raw_vec_iter() {
        let mut raw_vec: RawVec<i32> = RawVec::with_capacity(4).unwrap();
        unsafe {
            for i in 0..4 {
                ptr::write(raw_vec.ptr().as_ptr().add(i), i as i32);
            }
        }

        let mut iter = raw_vec.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_zen_vec_partialeq() {
        let mut vec1: ZenVec<u8> = ZenVec::new();
        let mut vec2: ZenVec<u8> = ZenVec::new();
        for i in 0..4 {
            vec1.push(i).unwrap();
            vec2.push(i).unwrap();
        }
        assert_eq!(vec1.get(0), Some(&0));
        assert!(vec1 == vec2, "PartialEq #1 fail for ZenVec");

        vec2.pop();
        assert!(vec1 != vec2, "PartialEq #2 fail for ZenVec");
        assert!(vec1 == [0, 1, 2, 3]);
    }

    #[test]
    fn test_box_allocation() {
        let value = ZenBox::new(42).unwrap();
        assert_eq!(*value, 42);
    }

    #[test]
    fn test_box_mutation() {
        let mut value = ZenBox::new(42).unwrap();
        *value = 100;
        assert_eq!(*value, 100);
    }

    #[test]
    fn test_box_drop() {
        let value = ZenBox::new(42).unwrap();
        drop(value);
        // Box should deallocate memory without issues
    }

    #[test]
    fn test_rc() {
        let value = 42;
        let rc1 = ZenRc::new(value).unwrap();
        assert_eq!(*rc1, value);
        assert_eq!(ZenRc::strong_count(&rc1), 1);

        let rc2 = ZenRc::clone(&rc1);
        assert_eq!(*rc2, value);
        assert_eq!(ZenRc::strong_count(&rc1), 2);
        assert_eq!(ZenRc::strong_count(&rc2), 2);

        drop(rc1);
        assert_eq!(ZenRc::strong_count(&rc2), 1);

        drop(rc2);
        // Ensure no double free occurs
    }

    #[test]
    fn test_arc() {
        let value = 42;
        let arc1 = ZenArc::new(value).unwrap();
        assert_eq!(*arc1, value);
        assert_eq!(ZenArc::strong_count(&arc1), 1);

        let arc2 = ZenArc::clone(&arc1);
        assert_eq!(*arc2, value);
        assert_eq!(ZenArc::strong_count(&arc1), 2);
        assert_eq!(ZenArc::strong_count(&arc2), 2);

        drop(arc1);
        assert_eq!(ZenArc::strong_count(&arc2), 1);

        drop(arc2);
        // Ensure no double free occurs
    }

    #[test]
    fn test_zen_cow() {
        let cow1 = ZenCow::new(42).unwrap();
        assert_eq!(*cow1.as_ref(), 42);

        let mut cow2 = cow1.clone();
        assert_eq!(*cow2.as_ref(), 42);

        *cow2.get_mut() = 100;
        assert_eq!(*cow2.as_ref(), 100);
        assert_eq!(*cow1.as_ref(), 42); // Ensure cow1 is unchanged
    }

    #[test]
    fn test_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        
        assert_send_sync::<ZenBox<u32>>();
        assert_send_sync::<ZenRc<u32>>();
        assert_send_sync::<ZenArc<u32>>();
        assert_send_sync::<ZenCow<u32>>();
        assert_send_sync::<ZenVec<u32>>();
    }
}
