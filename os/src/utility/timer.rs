// use core::arch::{asm, global_asm};
// use riscv::register::sie;
// use crate::utility::HARTID;
//
// pub const CLINT:usize=0x2000000;
// pub static CLINT_MTIMECMP:usize=CLINT+0x4000+8*HARTID;
// pub const CLINT_MTIME:usize=CLINT+0xbff8;
// pub const INTERVAL:usize=1000000;
//
// global_asm!(include_str!( "timer.S"));
//
// extern "C"{
//     fn timervec();
// }
//
// //todo unfinished timer_init
// pub unsafe fn init_timer(){
//     *(CLINT_MTIMECMP as *mut usize)= get_time()+INTERVAL;
//     let timervec_ptr=timervec as *usize;
//     let mut mstatus:usize;
//     let mut mie:usize;
//     asm!(
//         "csrw mtvec, {timerve_ptr}",
//         timervec_ptr=in(reg) timervec_ptr,
//         "csrr {mstatus}, mstatus",
//         mstatus=inout(reg) mstatus,
//     );
//     mstatus|= (1 as usize)<<3;
//     asm!(
//     "csrw mstatus, {mstatus}",
//     mstatus = in(reg) mstatus,
//     "csrr {mie},mie",
//     mie=inout(reg) mie
//     );
//     mie|=(1 as usize)<<7;
//     asm!(
//     "csrw mie,{mie}",
//     mie =in(reg) mie
//     )
// }
//
// pub unsafe fn get_time()->usize{
//     *(CLINT_MTIME as *usize)}
//
//
// pub unsafe fn reset_timer(){
//     *(CLINT_MTIMECMP as *mut usize)= get_time()+INTERVAL;
// }