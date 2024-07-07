use crate::zen_string::zen_ascii_char::ZenAsciiChar;
use crate::zen_vec::zen_vec::ZenVec;

pub enum ZenAsciiStringError {
    InvalidStr,
    AllocationError(core::alloc::AllocError),
}

pub struct ZenAsciiString {
    vec: ZenVec<ZenAsciiChar>,
}

impl ZenAsciiString {
    #[inline]
    pub fn new() -> Self {
        Self { vec: ZenVec::new() }
    }

    pub fn with_capacity(capacity: usize) -> Result<Self, ZenAsciiStringError> {
        match ZenVec::with_capacity(capacity) {
            Ok(vec) => Ok(Self { vec }),
            Err(alloc_err) => Err(ZenAsciiStringError::AllocationError(alloc_err)),
        }
    }

    pub fn from_str(s: &str) -> Result<Self, ZenAsciiStringError> {
        let mut obj = Self::with_capacity(s.len())?;
        for c in s.chars() {
            if let Some(ascii_char) = ZenAsciiChar::new(c) {
                obj.vec.push(ascii_char);
            } else {
                return Err(ZenAsciiStringError::InvalidStr);
            }
        }
        Ok(obj)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.vec.cap()
    }

    #[inline]
    pub fn pop(&mut self) -> Option<ZenAsciiChar> {
        self.vec.pop()
    }

    #[inline]
    pub fn push(&mut self, c: ZenAsciiChar) {
        self.vec.push(c);
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}
