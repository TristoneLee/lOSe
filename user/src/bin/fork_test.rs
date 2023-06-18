#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::{exec, fork, wait, yield_};

#[no_mangle]
fn main() -> i32 {
    println!("Fork test:");
    if fork() == 0 {
        println!("This is children")
    } else {
        println!("This is parent")
    }
    0
}
