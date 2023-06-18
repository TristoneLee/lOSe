use core::arch::global_asm;
use crate::trap::trap_return;
global_asm!(include_str!("switch.S"));

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

extern "C"{
    fn __switch(src:*const Context,des: *const Context);
}

pub unsafe fn cxt_switch(src: &Context, des: &Context){
    let _src=&src as *const Context;
    let _des=&des as *const Context;
    __switch(_src,_des);
}

