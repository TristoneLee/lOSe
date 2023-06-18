use core::arch::global_asm;
use crate::trap::trap_return;

global_asm!(include_str!("switch.S"));

#[repr(C)]
pub struct Context {
    ra:usize,
    sp:usize,
    a:[usize;12]
}

impl Context {
    pub fn new()->Self{
        Context{
            ra:0,
            sp:0,
            a:[0;12]
        }
    }

    pub fn goto_trap_return(kstack_ptr: usize) -> Self {
        Self {
            ra: trap_return as usize,
            sp: kstack_ptr,
            a: [0; 12],
        }
    }}



pub unsafe fn cxt_switch(src: *mut Context, des: *const Context){
    extern "C"{
        fn __switch(src:*mut Context,des: *const Context);
    }
    __switch(src,des);
}

