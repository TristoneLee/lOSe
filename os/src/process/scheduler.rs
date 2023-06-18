use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::Vec;
use lazy_static::lazy_static;
use xmas_elf::dynamic::Tag::Null;
use crate::loader::{get_app_data, get_app_data_by_name};
use crate::mm::pagetable::PageTable;
use crate::process::context::{Context, cxt_switch};
use crate::sync::cell::UPSafeCell;
use crate::process::process::{Process, ProcessStatus};
use crate::process::process::ProcessStatus::Dead;
use crate::trap::trap_context::TrapContext;

pub struct Scheduler {
    available_queue: Vec<Arc<Process>>,
    cur_prc: Option<Arc<Process>>,
    scheduler_cxt: Context,
}

pub fn cur_trap_cxt() -> &'static mut TrapContext {
    SCHEDULER.exclusive_access().cur_prc.unwrap().get_trap_cxt()
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

    pub fn get_cur_pg_table() -> & PageTable{
        & SCHEDULER.exclusive_access().cur_prc.unwrap().page_table
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
    pub fn kernel_yield() {
        let mut scheduler = SCHEDULER.exclusive_access();
        let cur_prc = &scheduler.cur_prc;
        assert_ne!(cur_prc, None);
        let cur_prc = cur_prc.unwrap();
        let cur_cxt_ptr = &cur_prc.context;
        prc.status = ProcessStatus::Ready;
        scheduler.push_prc(cur_prc);
        let scheduler_cxt_ptr = &scheduler.scheduler_cxt;
        unsafe {
            cxt_switch(cur_cxt_ptr, scheduler_cxt_ptr);
        }
    }

    //todo a lot to do initproc
    pub fn kernel_exit(exit_code: i32) {
        let scheduler = SCHEDULER.exclusive_access();
        let prc = &scheduler.cur_prc;
        assert_ne!(prc, None);
        let mut prc = prc.unwrap();
        let pid = prc.pid;
        //todo guanji
        prc.status = ProcessStatus::Dead;
        prc.exit_code = exit_code;
        prc.children.clear();
        prc.frame_recycle();
        let scheduler_cxt_ptr = &scheduler.scheduler_cxt;
        let null_cxt = &Context::new();
        unsafe {
            cxt_switch(null_cxt, scheduler_cxt_ptr);
        }
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
            |(_, p)| {
                (p.pid == pid as usize || pid == -1) && p.status == Dead
            }
        );
        if let Some((idx, _)) = pair {
            let child = prc.children.remove(idx);
            assert_eq!(Arc::strong_count(&child), 1);
            let found_pid = child.pid;
            let exit_code = child.exit_code;
            //todo can or not use pagetable of current process
            let exit_code_pa = prc.page_table.translate_va(exit_code_ptr as usize) as *mut i32;
            unsafe {
                *exit_code_pa = exit_code
            }
            found_pid as isize
        }
        -2
    }

    pub fn kernel_getpid() -> usize {
        SCHEDULER.exclusive_access().get_pid()
    }

    pub fn kernel_fork() -> isize {
        let mut scheduler = &SCHEDULER.exclusive_access();
        let cur_prc = scheduler.cur_prc.unwrap();
        let mut new_prc = cur_prc.clone();
        new_prc.parent = Option::from(Arc::downgrade(&cur_prc));
        cur_prc.children.push(new_prc.clone());
        let new_pid = new_prc.pid;
        let trap_cxt = new_prc.get_trap_cxt();
        trap_cxt.x[10] = 0;
        scheduler.push_prc(new_prc);
        new_pid as isize
    }

    pub fn kernel_exec(path: *const u8) -> isize {
        let mut scheduler = &SCHEDULER.exclusive_access();
        let mut cur_prc = scheduler.cur_prc.unwrap();
        let path = cur_prc.page_table.translated_str(path);
        if let Some(data) = get_app_data_by_name(path.as_str()) {
            cur_prc.exec(data);
            0
        } else {
            -1
        }
    }
}
