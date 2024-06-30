use core::alloc::Layout;
use core::ptr::NonNull;
use alloc::alloc::handle_alloc_error;

pub struct Box<T> {
    ptr: NonNull<T>,
}

impl<T> Box<T> {
    pub fn new(value: T) -> Self {
        let layout = Layout::new::<T>();
        let ptr = match unsafe { System.allocate(layout) } {
            Ok(non_null_ptr) => non_null_ptr.cast(),
            Err(_) => handle_alloc_error(layout),
        };

        unsafe {
            ptr::write(ptr.as_ptr(), value);
        }

        Self { ptr }
    }

    pub fn into_inner(self) -> T {
        let value = unsafe { ptr::read(self.ptr.as_ptr()) };
        core::mem::forget(self);
        value
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        let layout = Layout::new::<T>();
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            System.deallocate(self.ptr.cast(), layout);
        }
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}
