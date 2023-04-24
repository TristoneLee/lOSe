use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use xmas_elf::dynamic::Tag::Null;
use crate::process::context::{Context, cxt_switch};
use crate::sync::cell::UPSafeCell;
use crate::process::process::{Process, ProcessStatus};
use crate::process::process::ProcessStatus::Dead;

pub struct Scheduler {
    available_queue: Vec<Arc<Process>>,
    cur_prc: Option<Arc<Process>>,
    scheduler_cxt: Context,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            available_queue: vec![],
            cur_prc: None,
            scheduler_cxt: Context::new(),
        }
    }

    pub fn push_prc(&mut self, prc: Arc<Process>) {
        self.available_queue.push(prc);
    }

    pub fn pop(&mut self) -> Option<Arc<Process>> {
        self.available_queue.pop()
    }

    pub fn get_pid(&self) -> usize {
        self.cur_prc.unwrap().pid
    }

    pub fn get_cur_prc(&mut self) -> &mut Option<Arc<Process>> {
        &mut self.cur_prc
    }
}

lazy_static! {
    pub static ref SCHEDULER: UPSafeCell<Scheduler> =unsafe { UPSafeCell::new(Scheduler::new()) };
}
pub fn run() {
    loop {
        let mut scheduler = SCHEDULER.exclusive_access();
        if let Some(prc) = scheduler.pop() {
            let scheduler_cxt_ptr = &scheduler.scheduler_cxt;
            let next_cxt_ptr = &prc.context;
            prc.status = ProcessStatus::Running;
            scheduler.cur_prc = Some(prc);
            drop(scheduler);
            unsafe {
                cxt_switch(scheduler_cxt_ptr, next_cxt_ptr);
            }
        }
    }
}

impl Scheduler {
    pub fn kernel_yield(&mut self) {
        let prc = &self.cur_prc;
        assert_ne!(prc, None);
        let prc = prc.unwrap();
        let cur_cxt_ptr = &prc.context;
        prc.status = ProcessStatus::Ready;
        self.push_prc(prc);
        let scheduler_cxt_ptr = &self.scheduler_cxt;
        unsafe {
            cxt_switch(cur_cxt_ptr, scheduler_cxt_ptr);
        }
    }

    pub fn kernel_exit(&mut self, exit_code: i32) {
        let prc = &self.cur_prc;
        assert_ne!(prc, None);
        let prc = prc.unwrap();
        let pid = prc.pid;
        //todo guanji
        prc.status = ProcessStatus::Dead;
        prc.exit_code = exit_code;
    }

    pub fn kernel_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
        let prc = &SCHEDULER.exclusive_access().cur_prc;
        assert_ne!(prc, None);
        let prc = prc.unwrap();
        if prc.children.is_empty() ||
            pid != -1 && !prc.children.iter()
                .any(|p| pid == -1 || pid as usize == p.pid) {
            return -1;
        }
        let pair = prc.children.iter().enumerate().find(
            |(_,p)|{
                (p.pid==pid as usize||pid==-1)&& p.status==Dead
            }
        );
        if let Some((idx,_))=pair{
            let child = prc.children.remove(idx);
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid=child.pid;
            let exit_code=child.exit_code
            //todo can or not use pagetable of current process
            let exit_code_pa=prc.page_table.translate_va(exit_code_ptr as usize) as *mut i32;
            unsafe{
                *exit_code_pa=exit_code
            }
            found_pid as isize
        }
        -2
    }

    pub fn kernel_getpid() -> usize {
        SCHEDULER.exclusive_access().get_pid()
    }

    pub fn kernel_fork() {}

    pub fn kernel_exec() {}
}