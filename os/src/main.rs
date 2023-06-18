#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use riscv::register::*;
use crate::{io, mm, process, trap};

global_asm!(include_str!("start.asm"));

#[no_mangle]
unsafe fn rust_start() -> ! {
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);

    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    asm!("csrr tp, mhartid");

    //todo init_timer

    asm!(
    "csrw mideleg, {mideleg}", // some bits could not be set by this method
    "csrw medeleg, {medeleg}",
    "mret",
    medeleg = in(reg) !0,
    mideleg = in(reg) !0,
    options(noreturn),
    );
}

#[no_mangle]
extern "C" fn rust_main() {
    trap::init();
    io::init();
    mm::init();
    process::scheduler::run();
}
