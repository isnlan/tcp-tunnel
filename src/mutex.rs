use std::fmt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

#[derive(Debug)]
pub(crate) struct Mutex<T: ?Sized>(PhantomData<std::sync::Mutex<T>>, parking_lot::Mutex<T>);

#[derive(Debug)]
pub(crate) struct MutexGuard<'a, T: ?Sized>(
    PhantomData<std::sync::MutexGuard<'a, T>>,
    parking_lot::MutexGuard<'a, T>,
);

impl<T> Mutex<T> {
    #[inline]
    pub(crate) fn new(t: T) -> Mutex<T> {
        Mutex(PhantomData, parking_lot::Mutex::new(t))
    }

    #[inline]
    pub(crate) const fn const_new(t: T) -> Mutex<T> {
        Mutex(PhantomData, parking_lot::const_mutex(t))
    }

    #[inline]
    pub(crate) fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard(PhantomData, self.1.lock())
    }

    #[inline]
    pub(crate) fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.1
            .try_lock()
            .map(|guard| MutexGuard(PhantomData, guard))
    }

    #[inline]
    pub(crate) fn get_mut(&mut self) -> &mut T {
        self.1.get_mut()
    }
}

impl<'a, T: ?Sized> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.1.deref()
    }
}

impl<'a, T: ?Sized> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.1.deref_mut()
    }
}

impl<'a, T: ?Sized + fmt::Display> fmt::Display for MutexGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.1, f)
    }
}
