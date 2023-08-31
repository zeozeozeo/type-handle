use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::{mem, ptr};

/// Convert any reference into any other.
#[inline]
unsafe fn transmute_ref<FromT, ToT>(from: &FromT) -> &ToT {
    debug_assert_eq!(mem::size_of::<FromT>(), mem::size_of::<ToT>());
    &*(from as *const FromT as *const ToT)
}

/// Convert any mutable reference into any other.
#[inline]
pub(crate) unsafe fn transmute_ref_mut<FromT, ToT>(from: &mut FromT) -> &mut ToT {
    debug_assert_eq!(mem::size_of::<FromT>(), mem::size_of::<ToT>());
    &mut *(from as *mut FromT as *mut ToT)
}

pub struct Handle<T>(
    T,
    // `*const` is needed to prevent automatic Send and Sync derivation if T implements Send and Sync.
    PhantomData<*const ()>,
);

impl<T> AsRef<Handle<T>> for Handle<T> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T> From<T> for Handle<T> {
    fn from(t: T) -> Self {
        Self::from_instance(t)
    }
}

impl<T> Handle<T> {
    /// Wrap a struct instance into a handle.
    #[inline]
    #[must_use]
    pub fn from_instance(t: T) -> Self {
        Handle(t, PhantomData)
    }

    /// Wrap a struct reference into a handle.
    #[inline]
    #[must_use]
    pub fn from_ref(t: &T) -> &Self {
        unsafe { transmute_ref(t) }
    }

    /// Wrap a mutable struct reference into a mutable handle.
    #[inline]
    #[must_use]
    pub fn from_ref_mut(t: &mut T) -> &mut Self {
        unsafe { transmute_ref_mut(t) }
    }

    /// Wrap a const pointer into a const handle pointer.
    #[inline]
    #[must_use]
    pub fn from_ptr(tp: *const T) -> *const Self {
        tp as _
    }

    /// Wrap a mut pointer into a mut handle pointer.
    #[inline]
    #[must_use]
    pub fn from_ptr_mut(tp: *mut T) -> *mut Self {
        tp as _
    }

    /// Replaces the instance with the one from this Handle, and returns the replaced one
    /// wrapped in a Handle without dropping either one.
    #[inline]
    #[must_use]
    pub fn replace(mut self, t: &mut T) -> Self {
        mem::swap(&mut self.0, t);
        self
    }

    /// Consumes the wrapper and returns the wrapped type.
    #[inline]
    #[must_use]
    pub fn into_instance(mut self) -> T {
        let r = mem::replace(&mut self.0, unsafe { mem::zeroed() });
        mem::forget(self);
        r
    }

    /// Returns a reference to the wrapped type.
    #[inline]
    #[must_use]
    pub fn instance(&self) -> &T {
        &self.0
    }

    /// Returns a mutable reference to the wrapped type.
    #[inline]
    #[must_use]
    pub fn instance_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T: Clone> Clone for Handle<T> {
    fn clone(&self) -> Self {
        Self::from_instance(self.0.clone())
    }
}

impl<T: PartialEq> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.instance().eq(other.instance())
    }
}

impl<T> Deref for Handle<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.instance()
    }
}

impl<T> DerefMut for Handle<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.instance_mut()
    }
}

#[cfg(feature = "send_sync")]
unsafe impl<T> Send for Handle<T> {}

#[cfg(feature = "send_sync")]
unsafe impl<T> Sync for Handle<T> {}

/// A wrapper type represented by a reference counted pointer to the wrapped type.
#[repr(transparent)]
pub struct RCHandle<T>(ptr::NonNull<T>);

impl<T> From<&RCHandle<T>> for RCHandle<T> {
    fn from(rch: &RCHandle<T>) -> Self {
        rch.clone().into()
    }
}

impl<T> AsRef<RCHandle<T>> for RCHandle<T> {
    fn as_ref(&self) -> &RCHandle<T> {
        self
    }
}

impl<T> RCHandle<T> {
    /// Create a reference counted handle from a pointer.
    ///
    /// Takes ownership of the object the pointer points to, does not increase the reference count.
    ///
    /// Returns [`None`] if the pointer is `null`.
    #[inline]
    pub fn from_ptr(ptr: *mut T) -> Option<Self> {
        ptr::NonNull::new(ptr).map(Self)
    }

    /// Create a reference counted handle from a pointer.
    ///
    /// Shares ownership with the object the pointer points to, therefore increases the reference count.
    ///
    /// Returns [`None`] if the pointer is `null`.
    #[inline]
    pub fn from_unshared_ptr(ptr: *mut T) -> Option<Self> {
        ptr::NonNull::new(ptr).map(|ptr| {
            unsafe {
                let _ = ptr.as_ref();
            }
            Self(ptr)
        })
    }

    /// Create a reference to the wrapper from a reference to a pointer that points to the wrapped type.
    #[inline]
    pub fn from_unshared_ptr_ref(t: &*mut T) -> &Option<Self> {
        unsafe { transmute_ref(t) }
    }

    /// Create a reference counted handle from a mutable reference.
    ///
    /// Takes ownership of the referenced object.
    #[inline]
    pub fn from_ref(t: &mut T) -> Self {
        // references cannot be null, so it's safe to call unwrap_unchecked() here
        unsafe { Self::from_ptr(t).unwrap_unchecked() }
    }

    /// Returns the pointer to the handle.
    #[inline]
    pub fn as_ptr(&self) -> &ptr::NonNull<T> {
        &self.0
    }

    /// Returns a reference to the wrapped type.
    #[inline]
    pub fn as_ref(&self) -> &T {
        unsafe { self.0.as_ref() }
    }

    /// Returns a mutable reference to the wrapped type.
    #[inline]
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.0.as_mut() }
    }

    /// Consumes the wrapper and returns a pointer to the wrapped type.
    #[inline]
    pub fn into_ptr(self) -> *mut T {
        let ptr = self.0.as_ptr();
        mem::forget(self);
        ptr
    }
}

impl<T> Clone for RCHandle<T> {
    fn clone(&self) -> Self {
        let ptr = self.0;
        unsafe {
            let _ = ptr.as_ref();
        }
        Self(ptr)
    }
}

impl<T: PartialEq> PartialEq for RCHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T> Deref for RCHandle<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for RCHandle<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

#[cfg(feature = "send_sync")]
unsafe impl<T> Send for RCHandle<T> {}

#[cfg(feature = "send_sync")]
unsafe impl<T> Sync for RCHandle<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Thing {
        number: i32,
    }

    #[test]
    fn test_handle() {
        for num in 0..128 {
            let thing = Thing { number: num };
            let handle = Handle::from_instance(thing);
            assert!(handle.number == num && handle.instance().number == num);
        }
    }

    #[test]
    fn test_handle_mut() {
        for num in 0..128 {
            let thing = Thing { number: num };
            let mut handle = Handle::from_instance(thing);
            assert!(
                handle.number == num
                    && handle.instance().number == num
                    && handle.instance_mut().number == num
            );
            let new_num = num * 5;
            handle.number = new_num;
            assert!(handle.number == new_num);
        }
    }

    #[test]
    fn test_rchandle() {
        for num in 0..128 {
            let mut thing = Thing { number: num };
            let mut rch = RCHandle::from_ptr(&mut thing).unwrap();
            assert!(rch.number == num && rch.as_ref().number == num && rch.as_mut().number == num);
            let new_num = num * 6;
            rch.number = new_num;
            assert!(rch.number == new_num);

            let mut rch = RCHandle::from_ref(&mut thing);
            rch.number = 11;
            assert!(rch.number == 11);
        }
    }
}
