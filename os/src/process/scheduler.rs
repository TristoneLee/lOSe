use alloc::collections::VecDeque;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use core::borrow::Borrow;
use core::ops::Deref;

use lazy_static::lazy_static;
use xmas_elf::dynamic::Tag::Null;
use crate::io::print;

use crate::loader::{get_app_data, get_app_data_by_name};
use crate::mm::pagetable::PageTable;
use crate::println;
use crate::process::context::{Context, cxt_switch};
use crate::process::process::{INITPROC, Process, ProcessStatus, ProcessWrapper};
use crate::process::process::ProcessStatus::Dead;
use crate::sync::cell::Mutex;
use crate::trap::trap_context::TrapContext;

pub struct Scheduler {
    available_queue: Vec<Arc<ProcessWrapper>>,
    cur_prc: Option<Arc<ProcessWrapper>>,
    scheduler_cxt: Context,
}

impl Scheduler {
    pub fn new() -> Self {
        println!("Scheduler online");
        Scheduler {
            available_queue: Vec::new(),
            cur_prc: None,
            scheduler_cxt: Context::new(),
        }
    }

    pub fn take_current_prc(&mut self) -> Option<Arc<ProcessWrapper>> {
        self.cur_prc.take()
    }

    pub fn current_prc(&self) -> Option<Arc<ProcessWrapper>> {
        self.cur_prc.as_ref().map(Arc::clone)
    }

    pub fn push_prc(&mut self, prc: Arc<ProcessWrapper>) {
        self.available_queue.push(prc);
    }

    pub fn pop(&mut self) -> Option<Arc<ProcessWrapper>> {
        Some(self.available_queue.remove(0))
    }

    pub fn get_cur_pid() -> usize {
        SCHEDULER.lock().current_prc().unwrap().inner().pid
    }

    pub fn get_cur_token() -> usize {
        SCHEDULER.lock().current_prc().unwrap().inner().page_table.token()
    }

    // pub fn get_cur_pg_table() -> & 'static PageTable{
    //     & SCHEDULER.lock().current_prc().unwrap().page_table
    // }

    pub fn get_cur_trap_cxt() -> &'static mut TrapContext {
        SCHEDULER.lock().current_prc().unwrap().inner().get_trap_cxt()
    }
}

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> =unsafe { Mutex::new(Scheduler::new()) };
}

pub fn run() {
    println!("Begin scheduling!");
    loop {
        let mut scheduler = SCHEDULER.lock();
        if let Some(prc) = scheduler.pop() {
            let mut prc_inner = prc.inner();
            let scheduler_cxt_ptr = &mut scheduler.scheduler_cxt as *mut Context;
            let next_cxt_ptr = &prc_inner.context as *const Context;
            // println!("next prc {}", prc.pid);
            prc_inner.status = ProcessStatus::Running;
            drop(prc_inner);
            scheduler.cur_prc = Some(prc);
            drop(scheduler);
            unsafe {
                cxt_switch(scheduler_cxt_ptr, next_cxt_ptr);
            }
        }
    }
}

impl Scheduler {
    pub fn kernel_yield() {
        let mut scheduler = SCHEDULER.lock();
        let cur_prc = scheduler.current_prc().unwrap();
        let mut cur_prc_inner = cur_prc.inner();
        let cur_cxt_ptr = &mut cur_prc_inner.context as *mut Context;
        let scheduler_cxt_ptr = &scheduler.scheduler_cxt as *const Context;
        cur_prc_inner.status = ProcessStatus::Ready;
        // println!("Process {} yield.", cur_prc_inner.pid);
        drop(cur_prc_inner);
        scheduler.push_prc(cur_prc);
        // println!("[After yield] Have {} process in scheduler the first is {}.", scheduler.available_queue.len(), scheduler.available_queue.first().unwrap().pid);
        drop(scheduler);
        unsafe {
            cxt_switch(cur_cxt_ptr, scheduler_cxt_ptr);
        }
    }

    pub fn kernel_exit(exit_code: i32) {
        let scheduler = SCHEDULER.lock();
        let cur_prc = scheduler.current_prc().unwrap();
        let mut cur_prc_inner = cur_prc.inner();
        let pid = cur_prc_inner.pid;
        cur_prc_inner.status = ProcessStatus::Dead;
        cur_prc_inner.exit_code = exit_code;
        {
            let mut initproc_inner = INITPROC.inner();
            for child in cur_prc_inner.children.iter() {
                child.inner().parent = Some(Arc::downgrade(&INITPROC));
                initproc_inner.children.push(child.clone());
            }
        }
        cur_prc_inner.children.clear();
        cur_prc_inner.frame_recycle();
        let scheduler_cxt_ptr = &scheduler.scheduler_cxt as *const Context;
        drop(scheduler);
        drop(cur_prc_inner);
        let null_cxt = &mut Context::new();
        unsafe {
            cxt_switch(null_cxt, scheduler_cxt_ptr);
        }
    }

    pub fn kernel_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        let scheduler = SCHEDULER.lock();
        let cur_prc = scheduler.current_prc().unwrap();
        let mut cur_prc_inner = cur_prc.inner();
        // println!("Process {} waitpid.", cur_prc_inner.pid);
        if cur_prc_inner.children.is_empty() ||
            pid != -1 && !cur_prc_inner.children.iter()
                .any(|p| pid == -1 || pid as usize == p.pid) {
            return -1;
        }
        let pair = cur_prc_inner.children.iter().enumerate().find(
            |(_, p)| {
                (p.pid == pid as usize || pid == -1) && p.inner().status == Dead
            }
        );
        if let Some((idx, _)) = pair {
            let child = cur_prc_inner.children.remove(idx);
            let child_inner = child.inner();
            // assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child_inner.pid;
            let exit_code = child_inner.exit_code;
            //todo can or not use pagetable of current process
            let exit_code_pa = cur_prc_inner.page_table.translate_va(exit_code_ptr as usize).unwrap() as *mut i32;
            unsafe {
                *exit_code_pa = exit_code
            }
            return found_pid as isize;
        }
        -2
    }

    pub fn kernel_getpid() -> usize {
        Scheduler::get_cur_pid()
    }

    pub fn kernel_fork() -> isize {
        let mut scheduler = SCHEDULER.lock();
        let cur_prc = scheduler.current_prc().unwrap();
        let mut cur_prc_inner = cur_prc.inner();
        println!("Process {} fork.", cur_prc_inner.pid);
        let mut new_prc_inner = Process::clone(cur_prc_inner.borrow());
        new_prc_inner.parent = Option::from(Arc::downgrade(&cur_prc));
        let trap_cxt = new_prc_inner.get_trap_cxt();
        trap_cxt.x[10] = 0;
        let new_prc = Arc::new(ProcessWrapper::new(new_prc_inner));
        let new_pid = new_prc.pid;
        cur_prc_inner.children.push(new_prc.clone());
        scheduler.push_prc(new_prc);
        new_pid as isize
    }

    pub fn kernel_exec(path: *const u8) -> isize {
        let mut scheduler = &SCHEDULER.lock();
        let cur_prc = scheduler.current_prc().unwrap();
        let mut cur_prc_inner = cur_prc.inner();
        let path = cur_prc_inner.page_table.translated_str(path);
        if let Some(data) = get_app_data_by_name(path.as_str()) {
            cur_prc_inner.exec(data);
            0
        } else {
            -1
        }
    }
}
