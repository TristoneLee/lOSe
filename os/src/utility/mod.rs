use core::arch::asm;
use lazy_static::lazy_static;

pub mod panic;
pub mod recycle_counter;
pub mod timer;

pub static HARTID:usize = get_hartid();

pub fn get_hartid() -> usize {
    let hartid:usize;
    unsafe {
        asm!("cssr {0}, mhartid", in(reg)hartid);
    }
    hartid
}