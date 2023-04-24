#![no_std]
#![no_main]

use core::arch::{asm, global_asm};

global_asm!(include_str!("start.asm"));

#[no_mangle]
fn lboot() -> ! {
    unsafe {
        asm!(
        "pmpaddr0"
        "mret"
        )
    }
}