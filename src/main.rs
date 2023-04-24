#![no_std]
#![no_main]

extern crate alloc;

mod utility;
mod mm;
mod io;
mod process;
mod sync;
mod trap;
mod syscall;

fn main() {
    println!("Hello, world!");
}
