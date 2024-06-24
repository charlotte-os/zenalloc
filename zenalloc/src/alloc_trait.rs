use core::ptr::NonNull;

/// Error type for allocation failures.
#[derive(Debug)]
pub struct AllocError;

/// A trait for memory allocation.
pub trait Alloc {
    /// Allocate memory for the given size.
    /// Returns a pointer to the allocated memory or an error if allocation fails.
    fn alloc(&self, size: usize) -> Result<NonNull<u8>, AllocError>;

    /// Deallocate the memory referenced by the given pointer and size.
    /// This function never panics.
    fn dealloc(&self, ptr: NonNull<u8>, size: usize);

    /// Reallocate memory for the given pointer and size.
    /// Returns a pointer to the newly allocated memory or an error if reallocation fails.
    fn realloc(&self, ptr: NonNull<u8>, new_size: usize) -> Result<NonNull<u8>, AllocError>;

    /// Allocate zero-initialized memory for the given size.
    /// Returns a pointer to the allocated memory or an error if allocation fails.
    fn alloc_zeroed(&self, size: usize) -> Result<NonNull<u8>, AllocError> {
        let ptr = self.alloc(size)?;
        unsafe { core::ptr::write_bytes(ptr.as_ptr(), 0, size) };
        Ok(ptr)
    }
}

/// A simple allocator for demonstration purposes.
pub struct SimpleAllocator;

const BUFFER_SIZE: usize = 1024 * 1024; // 1 MiB buffer
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
static mut NEXT: usize = 0;

unsafe impl Alloc for SimpleAllocator {
    fn alloc(&self, size: usize) -> Result<NonNull<u8>, AllocError> {
        unsafe {
            if NEXT + size <= BUFFER_SIZE {
                let ptr = &mut BUFFER[NEXT] as *mut u8;
                NEXT += size;
                NonNull::new(ptr).ok_or(AllocError)
            } else {
                Err(AllocError)
            }
        }
    }

    fn dealloc(&self, _ptr: NonNull<u8>, _size: usize) {
        // No-op for this simple allocator
    }

    fn realloc(&self, ptr: NonNull<u8>, new_size: usize) -> Result<NonNull<u8>, AllocError> {
        let new_ptr = self.alloc(new_size)?;
        unsafe {
            core::ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), new_size);
        }
        Ok(new_ptr)
    }
}
