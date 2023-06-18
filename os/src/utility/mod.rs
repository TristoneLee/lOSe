use core::arch::asm;
use lazy_static::lazy_static;
use super::sync::cell::Mutex;

pub mod panic;
pub mod recycle_counter;
pub mod timer;

// lazy_static!(
//     pub static ref HARTID: Mutex<usize> =Mutex::new(get_hartid());
// );
//
//
// pub fn get_hartid() -> usize {
//     let hartid:usize=0;
//     unsafe {
//         asm!("cssr {0}, mhartid", in(reg)hartid);
//     }
//     hartid
// }