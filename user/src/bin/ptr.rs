#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::console::{print};

static mut fnd: [u8; 4096] = [0 as u8; 4096];
static mut fnd2: [u8; 4096] = [0 as u8; 4096];

#[no_mangle]
unsafe fn main() -> i32 {
    let mut stack0 = [0 as u8; 10];
    let mut stack = [0 as u8; 2050];
    let mut stack2 = [0 as u8; 10];
    for i in 0..2050 {
        fnd[i] = (i % 255) as u8;
    }
    stack[2049] = 1;
    for i in 0..2050 {
        stack[i] = fnd[i];
    }
    println!("{}", fnd[65]);
    for i in 0..2050 {
        fnd2[i] = (i % 255) as u8;
    }
    stack[2049] = 1;
    for i in 0..2050 {
        stack[i] = fnd2[i];
    }
    stack2[0] = fnd[66];
    stack2[1] = fnd[69];
    println!("{}", fnd2[65]);
    println!("{}", *((&stack[0] as *const u8).add(2050)));
    println!("{}", *((&stack[0] as *const u8).sub(1)));
    0
}
