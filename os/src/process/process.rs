use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::borrow::BorrowMut;
use core::cell::RefMut;
use core::cmp::min;
use lazy_static::lazy_static;
use riscv::register::satp;
use crate::io::print;
use crate::loader::get_app_data_by_name;
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::map_area::{MAP_PERM_R, MAP_PERM_U, MAP_PERM_W, MAP_PERM_X, MapArea, MapType};
use crate::mm::pagetable::PageTable;
use crate::mm::{addr_to_page_num, MEMORY_END, page_num_to_addr, PAGE_SIZE, PhysPageNum, read_frame, TRAMPOLINE, TRAP_CONTEXT, USER_STACK_SIZE, VirAddr, VirPageNum};
use crate::mm::kernel_space::{KERNEL_SPACE, kernel_stack_top};
use crate::mm::map_area::MapType::Framed;
use crate::println;
use crate::process::context::Context;
use crate::process::process::ProcessStatus::{Ready};
use crate::process::scheduler::SCHEDULER;
use crate::sync::cell::{Mutex, MutexGuard};
use crate::trap::trap_context::TrapContext;
use crate::trap::trap_handler;
use crate::utility::recycle_counter::RecycleCounter;

lazy_static!(
    pub static ref PID_ALLOCATOR: Mutex<RecycleCounter> = Mutex::new(RecycleCounter::new(usize::MAX - 1));
);

#[derive(PartialEq)]
pub enum ProcessStatus {
    Running,
    Ready,
    Dead,
}

pub struct ProcessWrapper{
    pub(crate) pid:usize,
    inner: Mutex<Process>
}

impl ProcessWrapper{
    pub fn new(prc: Process)->Self{
        ProcessWrapper{
            pid:prc.pid,
            inner: Mutex::new(prc)
        }
    }

    pub fn inner(&self) -> MutexGuard< Process> {
        self.inner.lock()
    }
}

pub struct Process {
    pub pid: usize,
    pub exit_code: i32,
    pub context: Context,
    pub status: ProcessStatus,
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
    pub parent: Option<Weak<ProcessWrapper>>,
    pub children: Vec<Arc<ProcessWrapper>>,
    pub trap_context_ppn: PhysPageNum,
}

impl Process {
    fn area_loading(&mut self, area: &mut MapArea, data: Option<&[u8]>) {
        if data.is_none() { return; }
        let data=data.unwrap();
        let mut ptr: usize = 0;
        let mut vpn = area.start;
        while ptr < data.len(){
            let src = &data[ptr..min(ptr + PAGE_SIZE, data.len())];
            let des = &mut read_frame(*area.frame_mapping.get(&vpn).unwrap() as PhysPageNum)[..src.len()];
            des.copy_from_slice(src);
            ptr += PAGE_SIZE;
            vpn += 1;
        }
    }

    pub fn frame_recycle(&mut self) {
        for (_,area) in self.areas.iter().enumerate() {
            area.recycle();
        }
    }

    pub fn get_trap_cxt(&self) -> &'static mut TrapContext {
        unsafe {
            (page_num_to_addr(self.trap_context_ppn) as *mut TrapContext).as_mut().unwrap()
        }
    }

    pub fn load_trap_cxt_trampoline(&mut self) {
        let mut area=MapArea::new(
            TRAP_CONTEXT.into(),
            TRAMPOLINE.into(),
            MapType::Framed,
            MAP_PERM_R | MAP_PERM_W,
        );

        self.page_table.area_mapping(&mut area);
        self.areas.push(area);
        self.page_table.load_trampoline();
        self.trap_context_ppn = self.page_table.find_pte(addr_to_page_num(TRAP_CONTEXT)).unwrap().ppn();
    }

    pub fn load_elf(elf_data: &[u8]) -> Process {
        let pg_root = frame_alloc().unwrap();
        let mut page_table = PageTable::new(pg_root);
        let pid: usize;
        unsafe {
            pid = PID_ALLOCATOR.lock().alloc().unwrap();
        }
        KERNEL_SPACE.lock().kernel_stack_apply(pid);
        let mut process = Process {
            pid,
            exit_code: 0,
            context: Context::goto_trap_return(kernel_stack_top(pid)),
            status: Ready,
            page_table,
            areas: vec![],
            parent: None,
            children: vec![],
            trap_context_ppn: 0,
        };
        process.load_trap_cxt_trampoline();
        process.elf_parser(elf_data);
        process
    }

    pub fn exec(& mut self, elf_data: &[u8]) {
        self.frame_recycle();
        self.areas = vec![];
        let pg_root = frame_alloc().unwrap();
        self.page_table = PageTable::new(pg_root);
        self.load_trap_cxt_trampoline();
        self.elf_parser(elf_data);
    }

    pub fn clone(obj: &Self) -> Self {
        let mut pid: usize;
        unsafe {
            pid = PID_ALLOCATOR.lock().alloc().unwrap();
        }
        KERNEL_SPACE.lock().kernel_stack_apply(pid);
        let pg_root = frame_alloc().unwrap();
        let mut this = Process {
            pid,
            exit_code: 0,
            context: (Context::goto_trap_return(kernel_stack_top(pid))),
            status: Ready,
            page_table: (PageTable::new(pg_root)),
            areas: vec![],
            parent: None,
            children: vec![],
            trap_context_ppn: 0,
        };
        this.page_table.load_trampoline();
        for area in obj.areas.iter() {
            let mut cur_area = area.clone();
            this.page_table.area_mapping(&mut cur_area);
            this.areas.push(cur_area);
            for vpn in area.start..area.end {
                let src = obj.page_table.find_pte(vpn).unwrap().ppn();
                let dst = this.page_table.find_pte(vpn).unwrap().ppn();
                read_frame(dst).copy_from_slice(read_frame(src));
            }
        }
        this.trap_context_ppn = this.page_table.find_pte(addr_to_page_num(TRAP_CONTEXT)).unwrap().ppn();
        let trap_context = this.get_trap_cxt();
        trap_context.kernel_sp = kernel_stack_top(pid);
        this
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    pub fn elf_parser(&mut self,elf_data:&[u8]){
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let mut max_end_vpn: VirPageNum = 0;
        //map app memory area
        for i in 0..elf_header.pt2.ph_count() {
            let ph = elf.program_header(i).unwrap();
            if ph.get_type().unwrap() == xmas_elf::program::Type::Load {
                let start_va: VirAddr = ph.virtual_addr() as VirAddr;
                let end_va: VirAddr = (ph.virtual_addr() + ph.mem_size()) as VirAddr;
                let mut map_perm = MAP_PERM_U;
                let ph_flags = ph.flags();
                if ph_flags.is_read() {
                    map_perm |= MAP_PERM_R;
                }
                if ph_flags.is_write() {
                    map_perm |= MAP_PERM_W;
                }
                if ph_flags.is_execute() {
                    map_perm |= MAP_PERM_X;
                }
                let mut map_area = MapArea::new(
                    start_va,
                    end_va,
                    Framed,
                    map_perm
                );
                max_end_vpn = map_area.end;
                self.page_table.area_mapping(&mut map_area);
                self.area_loading(&mut map_area, Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]));
                self.areas.push(map_area);
            }
        }
        //map user_stack
        let user_stack_bottom = page_num_to_addr(max_end_vpn) + PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let mut user_stack_area = MapArea::new(
            user_stack_bottom,
            user_stack_top,
            Framed,
            MAP_PERM_U | MAP_PERM_R | MAP_PERM_W
        );
        println!("User stack range {:#x} to {:#x}",user_stack_bottom,user_stack_top);
        self.page_table.area_mapping(&mut user_stack_area);
        self.areas.push(user_stack_area);
        let trap_cxt = self.get_trap_cxt();
        *trap_cxt = TrapContext::app_init_context(
            elf.header.pt2.entry_point() as usize,
            user_stack_top,
            KERNEL_SPACE.lock().kernel_token(),
            kernel_stack_top(self.pid),
            trap_handler as usize,
        );
    }
}

lazy_static! {
    pub static ref INITPROC: Arc<ProcessWrapper> = Arc::new(ProcessWrapper::new(
        Process::load_elf( get_app_data_by_name("initproc").unwrap())
    ));
}
pub fn add_initproc() {
    SCHEDULER.lock().push_prc(INITPROC.clone());
    println!("Initproc loaded");
}
