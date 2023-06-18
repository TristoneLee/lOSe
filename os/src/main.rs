#![no_std]
#![no_main]
#![feature(naked_functions, asm_const, const_cmp, fn_align, inline_const)]
#![feature(panic_info_message)]

extern crate alloc;

mod loader;
mod trap;
mod syscall;
mod sync;
mod io;
mod mm;
mod utility;
mod process;

use core::arch::{asm, global_asm};
use riscv::register::*;
use crate::loader::list_apps;
use crate::process::process::add_initproc;
use crate::utility::timer::init_timer;

pub const BOOTLOADER_STACK_SIZE: usize = 0x10000;
pub const CPUS: usize = 1;

global_asm!(include_str!("link_app.S"));

#[link_section = ".bss.stack"]
static mut BOOTLOADER_STACK_SPACE: [[u8; BOOTLOADER_STACK_SIZE]; CPUS] =
    [[0; BOOTLOADER_STACK_SIZE]; CPUS];


#[naked]
#[no_mangle]
#[link_section = ".text.entry"]
unsafe extern "C" fn _start() {
    asm!(
    "la sp, {bootloader_stack}",
    "li t0, {bootloader_stack_size}",
    "csrr t1, mhartid",
    "addi t1, t1, 1",
    "mul t0, t0, t1",
    "add sp, sp, t0",
    "j {rust_start}",
    bootloader_stack = sym BOOTLOADER_STACK_SPACE,
    bootloader_stack_size = const BOOTLOADER_STACK_SIZE,
    rust_start = sym rust_start,
    options(noreturn),
    );
}

#[no_mangle]
unsafe fn rust_start() -> ! {
    mstatus::set_mpp(riscv::register::mstatus::MPP::Supervisor);
    mepc::write(rust_main as usize);

    satp::write(0);

    pmpaddr0::write(0x3fffffffffffffusize);
    pmpcfg0::write(0xf);

    asm!("csrr tp, mhartid");

    // init_timer();

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
    println!("mm init");
    list_apps();
    add_initproc();
    process::scheduler::run();
}
