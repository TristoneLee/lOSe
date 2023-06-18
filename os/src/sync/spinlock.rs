use core::sync::atomic::{AtomicBool, Ordering};
use crate::println;
use crate::process::scheduler::{SCHEDULER, Scheduler};

pub struct Spinlock {
    if_lock: AtomicBool,
}

impl Spinlock {
    pub fn lock(&self) {
        while self.if_lock.compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire).is_err() {
            Scheduler::kernel_yield();
        }
    }

    pub fn unlock(&self) {
        if self.if_lock.compare_exchange(true, false, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            return;
        } else { println! {"Alarm: attempt to unlock a free lock"}; }
    }


    pub const fn new() ->Self{
        Spinlock{
            if_lock:AtomicBool::new(false)
        }
    }
}