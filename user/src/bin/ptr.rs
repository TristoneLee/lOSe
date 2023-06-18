#![no_std]
#![no_main]

#[macro_use]
extern crate user_lib;

use user_lib::console::{print};

#[no_mangle]
fn main() -> i32 {
    let mut fnd=[0 as u8;1000];
    let mut slip=[0 as u8;24];
    let mut fnd2=[0 as u8;2000];
    let mut bubble=[0 as u8;10];
    let mut stack=[0 as u8;2050];
    for i  in 0..2050{
        fnd[i]=(i%255) as u8;
    }
    stack[2049]=1;
    for i in 0..2050{
        stack[i]=fnd[i];
    }
    println!("{}",fnd[65]);
    0
}

// char fnd[1000];
// char slip[24];
// char fnd2[2000];
// char bubble[10];
//
// int main(){
// char stack[2050];
// for(int i = 0; i < 2050; ++i) fnd[i] = i % 255;
// stack[2049] = 1;
// for(int i = 0; i < 2050; ++i) stack[i] = fnd[i];
// putchar(fnd[65]);
// }