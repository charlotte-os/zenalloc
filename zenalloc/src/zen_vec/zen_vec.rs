use crate::zen_vec::raw_vec::RawVec;
use core::{
    alloc::AllocError,
    ops::{Deref, DerefMut, Drop},
    ptr, slice,
};

#[derive(PartialEq)]
pub enum VecError {
    IndexOutOfBounds,
    AllocationError(AllocError),
}

pub struct ZenVec<T> {
    buf: RawVec<T>,
    len: usize,
}

impl<T> ZenVec<T> {
    pub fn new() -> Self {
        Self {
            buf: RawVec::new(),
            len: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Result<Self, core::alloc::AllocError> {
        Ok(Self {
            buf: RawVec::with_capacity(capacity)?,
            len: 0,
        })
    }

    pub fn ptr(&self) -> *mut T {
        self.buf.ptr().as_ptr()
    }

    pub fn cap(&self) -> usize {
        self.buf.capacity()
    }

    pub fn push(&mut self, elem: T) -> Result<(), VecError> {
        if self.len == self.cap() {
            if let Err(alloc_err) = self.buf.grow() {
                return Err(VecError::AllocationError(alloc_err));
            }
        }

        unsafe {
            ptr::write(self.ptr().add(self.len), elem);
        }

        /* Can't overflow, we'll OOM first. */
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.ptr().add(self.len))) }
        }
    }

    pub fn insert(&mut self, index: usize, elem: T) -> Result<(), VecError> {
        if index > self.len {
            return Err(VecError::IndexOutOfBounds);
        }

        if self.len == self.cap() {
            if let Err(alloc_err) = self.buf.grow() {
                return Err(VecError::AllocationError(alloc_err));
            }
        }

        unsafe {
            ptr::copy(
                self.ptr().add(index),
                self.ptr().add(index + 1),
                self.len - index,
            );
            ptr::write(self.ptr().add(index), elem);
        }

        self.len += 1;
        Ok(())
    }

    pub fn remove(&mut self, index: usize) -> Result<T, VecError> {
        if index >= self.len {
            return Err(VecError::IndexOutOfBounds);
        }

        self.len -= 1;

        unsafe {
            let result = ptr::read(self.ptr().add(index));
            ptr::copy(
                self.ptr().add(index + 1),
                self.ptr().add(index),
                self.len - index,
            );
            Ok(result)
        }
    }

    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr(), self.len) }
    }

    pub fn as_mut_slice(&self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr(), self.len) }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for ZenVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
        /* Deallocation is handled by RawVec */
    }
}

impl<T> Deref for ZenVec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T> DerefMut for ZenVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T: PartialEq> PartialEq<[T]> for ZenVec<T> {
    fn eq(&self, other: &[T]) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self.as_slice() == other
    }
}

impl<T, const N: usize> PartialEq<[T; N]> for ZenVec<T>
where
    [T; N]: PartialEq,
    T: core::cmp::PartialEq,
{
    fn eq(&self, other: &[T; N]) -> bool {
        if self.len() != N {
            return false;
        }

        self.as_slice() == other
    }
}

impl<T> PartialEq<ZenVec<T>> for ZenVec<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &ZenVec<T>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        self == other.as_slice()
    }
}
