use core::arch::global_asm;
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
}

extern "C"{
    fn __switch(src:*const Context,des: *const Context);
}

pub unsafe fn cxt_switch(src: &Context, des: &Context){
    let _src=&src as *const Context;
    let _des=&des as *const Context;
    __switch(_src,_des);
}

