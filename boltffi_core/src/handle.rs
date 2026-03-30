use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct HandleBox<T> {
    ptr: NonNull<T>,
}

impl<T> HandleBox<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        let boxed = Box::new(value);
        Self {
            ptr: unsafe { NonNull::new_unchecked(Box::into_raw(boxed)) },
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_non_null(&self) -> NonNull<T> {
        self.ptr
    }

    #[inline]
    pub fn into_raw(self) -> *mut T {
        let ptr = self.into_non_null();
        ptr.as_ptr()
    }

    #[inline]
    pub fn into_non_null(self) -> NonNull<T> {
        let ptr = self.ptr;
        core::mem::forget(self);
        ptr
    }

    #[inline]
    pub unsafe fn from_raw(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|pointer| Self { ptr: pointer })
    }

    #[inline]
    pub unsafe fn from_non_null(ptr: NonNull<T>) -> Self {
        Self { ptr }
    }
}

impl<T> AsRef<T> for HandleBox<T> {
    #[inline]
    fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> AsMut<T> for HandleBox<T> {
    #[inline]
    fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Deref for HandleBox<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for HandleBox<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> From<T> for HandleBox<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> Drop for HandleBox<T> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.ptr.as_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_roundtrip() {
        let handle = HandleBox::new(42u32);
        let ptr = handle.into_raw();
        let recovered = unsafe { HandleBox::from_raw(ptr) }.unwrap();
        assert_eq!(*recovered.as_ref(), 42);
    }

    #[test]
    fn handle_null() {
        let result: Option<HandleBox<u32>> = unsafe { HandleBox::from_raw(core::ptr::null_mut()) };
        assert!(result.is_none());
    }

    #[test]
    fn handle_non_null_roundtrip() {
        let handle = HandleBox::from(42u32);
        let ptr = handle.into_non_null();
        let recovered = unsafe { HandleBox::from_non_null(ptr) };
        assert_eq!(*recovered, 42);
    }

    #[test]
    fn handle_accessors_return_same_pointer() {
        let handle = HandleBox::new(42u32);
        assert_eq!(handle.as_ptr(), handle.as_non_null().as_ptr());
    }
}
