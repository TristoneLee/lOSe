use core::any::Any;
use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut, UnsafeCell};
use core::ops::{Deref, DerefMut};
use core::sync::atomic::Ordering;
use crate::println;
use crate::sync::spinlock::Spinlock;

pub struct Mutex<T> {
    lock: Spinlock,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for Mutex<T> {}
unsafe impl<T: Send> Send for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            inner: UnsafeCell::new(value),
            lock: Spinlock::new(),
        }
    }

    pub fn lock(& self)->MutexGuard<T> {
        self.lock.lock();
        MutexGuard::new(&self, &self.lock, unsafe{&mut *self.inner.get()})
    }
}



pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    lock: &'a Spinlock,
    value: &'a mut T,
}

impl<'a, T> MutexGuard<'a, T> {
    pub fn new(mutex: &'a Mutex<T>, lock: &'a Spinlock, value: &'a mut T) -> Self{
        MutexGuard {
            mutex,
            lock,
            value,
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.value
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}