use crate::process::scheduler::SCHEDULER;

pub fn sys_read(fd:usize, buf: *const u8, len:usize) ->isize{
    //todo
    0
}

pub fn sys_write(fd:usize, buf:*const u8,len:usize)->isize{
    //todo
    0
}

pub fn sys_exit(exit_code:i32)->!{
    SCHEDULER.exclusive_access().kernel_exit(exit_code)
}

pub fn sys_yield()->isize{
    SCHEDULER.exclusive_access().kernel_yield();
    0
}

pub fn sys_getpid()->isize{
    SCHEDULER.exclusive_access().get_pid() as isize
}

pub fn sys_fork()-> isize{
    //todo
    0
}

pub fn sys_get_time()->isize{
    //todo
    0
}

pub fn sys_exec(path: *const u8)->isize{
    //todo
    0
}

pub fn sys_waitpid(pid:isize,exit_code_ptr: *mut i32)->isize{
    0
    //todo
}

pub fn sys_open()->isize{
    0
    //todo
}