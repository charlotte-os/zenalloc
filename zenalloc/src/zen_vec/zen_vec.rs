use crate::zen_vec::raw_vec::RawVec;
use core::{
    alloc::AllocError,
    ops::{Deref, DerefMut, Drop},
    ptr, slice,
};

#[derive(Debug, PartialEq)]
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

    pub fn iter(&self) -> ZenVecIter<'_, T> {
        ZenVecIter {
            zen_vec: self,
            index: 0,
        }
    }

    pub fn iter_mut(&mut self) -> ZenVecIterMut<'_, T> {
        ZenVecIterMut {
            zen_vec: self,
            index: 0,
        }
    }
}

impl<T> Drop for ZenVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
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

pub struct ZenVecIter<'a, T> {
    zen_vec: &'a ZenVec<T>,
    index: usize,
}

impl<'a, T> Iterator for ZenVecIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.zen_vec.len() {
            unsafe {
                let item = &*self.zen_vec.ptr().add(self.index);
                self.index += 1;
                Some(item)
            }
        } else {
            None
        }
    }
}

pub struct ZenVecIterMut<'a, T> {
    zen_vec: &'a mut ZenVec<T>,
    index: usize,
}

impl<'a, T> Iterator for ZenVecIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.zen_vec.len() {
            unsafe {
                let item = &mut *self.zen_vec.ptr().add(self.index);
                self.index += 1;
                Some(item)
            }
        } else {
            None
        }
    }
}

unsafe impl<T: Send> Send for ZenVec<T> {}
unsafe impl<T: Sync> Sync for ZenVec<T> {}
