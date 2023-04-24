use core::borrow::BorrowMut;
use core::cell::{RefCell, RefMut, UnsafeCell};
use crate::sync::spinlock::Spinlock;

pub struct UPSafeCell<T>{
    // lock:Spinlock,
    inner: RefCell<T>
}

unsafe impl<T> Sync for UPSafeCell<T>{

}
//todo: guard
impl<T> UPSafeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
            // lock: Spinlock::new()
        }
    }

    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        // self.lock.lock();
        self.inner.borrow_mut()
    }

    // pub fn done_accessing(&mut self){
    //     self.lock.unlock()
    // }
}
