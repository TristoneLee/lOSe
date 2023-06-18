use crate::io::uart::uart_getchar;
use crate::mm::pagetable::PageTable;
use crate::{print, println};
use crate::process::scheduler::{SCHEDULER, Scheduler};
use crate::utility::timer::get_time;

const STDIN: usize = 0;
const STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        STDOUT => {
            let buffers = PageTable::from_token(Scheduler::get_cur_token()).translated_byte_buffer(buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    match fd {
        STDIN => {
            let mut c: usize;
            loop {
                unsafe {
                    c = uart_getchar() as usize;
                }
                if c == 0 {
                    Scheduler::kernel_yield();
                    continue;
                } else {
                    break;
                }
            }
            let ch = c as u8;
            let mut buffers = PageTable::from_token(Scheduler::get_cur_token()).translated_byte_buffer( buf, len);
            unsafe {
                buffers[0].as_mut_ptr().write_volatile(ch);
            }
            1
        }
        _ => {
            panic!("Unsupported fd in sys_read!");
        }
    }
}

pub fn sys_exit(exit_code: i32) -> ! {
    Scheduler::kernel_exit(exit_code);
    panic!("Unreachable in sys_exit!");
}

pub fn sys_yield() -> isize {
    Scheduler::kernel_yield();
    0
}

pub fn sys_getpid() -> isize {
    Scheduler::kernel_getpid() as isize
}

pub fn sys_fork() -> isize {
    Scheduler::kernel_fork()
}

pub fn sys_get_time() -> isize {
    unsafe{get_time() as isize}
}

pub fn sys_exec(path: *const u8) -> isize {
    Scheduler::kernel_exec(path)
}

pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    Scheduler::kernel_waitpid(pid, exit_code_ptr)
}

pub fn sys_open() -> isize {
    0
    //todo
}