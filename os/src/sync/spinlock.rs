#![no_main]
#![no_std]

use core::sync::atomic::{AtomicBool, Ordering};
use crate::io::print;
use crate::println;

pub(crate) struct Spinlock {
    if_lock: AtomicBool,
}

impl Spinlock {
    pub fn lock(&self) {
        while self.if_lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire) {}
    }

    pub fn unlock(&self) {
        if self.if_lock.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed) {
            return;
        } else { println! {"Alarm: attempt to unlock a free lock"}; }
    }


    pub(crate) fn new() ->Self{
        Spinlock{
            if_lock:AtomicBool::new(false)
        }
    }
}