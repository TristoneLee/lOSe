use core::arch::{asm, global_asm};
use riscv::register::{mtvec, sie, mscratch, mie, mstatus};
use crate::println;

pub const CLINT: usize = 0x2000000;
pub static CLINT_MTIMECMP: usize = CLINT + 0x4000;
pub const CLINT_MTIME: usize = CLINT + 0xbff8;
pub const INTERVAL: usize = 1000000;

global_asm!(include_str!( "timer.S"));

extern "C" {
    fn timervec();
}

#[link_section = ".bss.stack"]
pub static mut SCRATCH: [usize; 5] = [0; 5];

pub unsafe fn init_timer() {
    let mut hartid: usize = 0;
    unsafe {
        asm!("mv {hartid}, tp",
        hartid = out(reg)hartid);
    }
    assert_eq!(hartid, 0);
    reset_timer();
    let timervec_ptr = timervec as *mut usize;
    let scratch = &mut SCRATCH;
    scratch[3] = CLINT_MTIMECMP;
    scratch[4] = INTERVAL;
    mscratch::write(scratch.as_ptr() as usize);
    mtvec::write(timervec_ptr as usize, mtvec::TrapMode::Direct);
    mstatus::set_mie();
    mie::set_mtimer();
}

pub unsafe fn get_time() -> usize {
    (CLINT_MTIME as *const usize).read_volatile()
}


pub unsafe fn reset_timer() {
    *(CLINT_MTIMECMP as *mut usize) = get_time() + INTERVAL;
}