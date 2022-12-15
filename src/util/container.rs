use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[repr(transparent)]
pub struct Container<T> {
    storage: *mut std::ffi::c_void,
    _data: PhantomData<T>,
}

impl<T> Container<T> {
    pub fn is_inline() -> bool {
        std::mem::size_of::<T>() <= std::mem::size_of::<*mut std::ffi::c_void>()
    }

    pub fn new(value: T) -> Self {
        let storage = if Self::is_inline() {
            let mut storage = std::ptr::null_mut();

            unsafe {
                std::ptr::copy_nonoverlapping(
                    &value as *const _ as *const u8,
                    (&mut storage) as *mut _ as *mut u8,
                    std::mem::size_of::<T>(),
                )
            };

            std::mem::forget(value);

            storage
        } else {
            Box::into_raw(Box::new(value)) as _
        };

        Self {
            storage,
            _data: PhantomData,
        }
    }

    pub unsafe fn from_raw(storage: *mut std::ffi::c_void) -> Self {
        Self {
            storage,
            _data: PhantomData,
        }
    }

    pub unsafe fn from_raw_const(storage: *const std::ffi::c_void) -> Self {
        Self {
            storage: std::mem::transmute(storage),
            _data: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const T {
        if Self::is_inline() {
            (&self.storage) as *const _ as *const T
        } else {
            self.storage as *const T
        }
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        if Self::is_inline() {
            (&mut self.storage) as *mut _ as *mut T
        } else {
            self.storage as *mut T
        }
    }

    pub fn into_inner(self) -> T {
        let value = if Self::is_inline() {
            unsafe { std::ptr::read((&self.storage) as *const _ as *const T) }
        } else {
            unsafe { *Box::from_raw(self.storage as _) }
        };

        std::mem::forget(self);

        value
    }

    pub fn into_raw(self) -> *mut std::ffi::c_void {
        let value = self.storage;

        std::mem::forget(self);

        value
    }
}

impl<T> Drop for Container<T> {
    fn drop(&mut self) {
        if Self::is_inline() {
            unsafe { std::ptr::drop_in_place(self.as_mut_ptr()) };
        } else {
            unsafe { drop(Box::<T>::from_raw(self.storage as _)) }
        }
    }
}

impl<T> AsRef<T> for Container<T> {
    fn as_ref(&self) -> &T {
        unsafe { &*self.as_ptr() }
    }
}

impl<T> AsMut<T> for Container<T> {
    fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<T> Deref for Container<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for Container<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl<T> Clone for Container<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Container::new(self.as_ref().clone())
    }
}

impl<T> Debug for Container<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("data", self.as_ref())
            .field("_data", &self._data)
            .field("#is_inline", &Self::is_inline())
            .finish()
    }
}

unsafe impl<T> Send for Container<T> where T: Send {}
unsafe impl<T> Sync for Container<T> where T: Sync {}
