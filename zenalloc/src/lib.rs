#![no_std]

pub mod alloc_trait;
mod alloc_trait;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc_trait::{Alloc, SimpleAllocator};

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_alloc_zeroed() {
        let allocator = SimpleAllocator;
        let size = 1024;
        let ptr = allocator.alloc_zeroed(size).expect("Allocation failed");
        unsafe {
            for i in 0..size {
                assert_eq!(*ptr.as_ptr().add(i), 0);
            }
        }
        allocator.dealloc(ptr, size);
    }

    #[test]
    fn test_realloc() {
        let allocator = SimpleAllocator;
        let size = 1024;
        let ptr = allocator.alloc(size).expect("Allocation failed");

        let new_size = 2048;
        let new_ptr = allocator.realloc(ptr, new_size).expect("Reallocation failed");

        allocator.dealloc(new_ptr, new_size);
    }
}
