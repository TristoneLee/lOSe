use alloc::sync::{Arc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use core::cmp::min;
use lazy_static::lazy_static;
use riscv::register::satp;
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::map_area::{MAP_PERM_R, MAP_PERM_U, MAP_PERM_W, MAP_PERM_X, MapArea, MapType};
use crate::mm::pagetable::PageTable;
use crate::mm::{MEMORY_END, page_num_to_addr, PAGE_SIZE, PhysPageNum, read_frame, USER_STACK_SIZE, VirAddr, VirPageNum};
use crate::mm::kernel_space::KERNEL_SPACE;
use crate::mm::map_area::MapType::Framed;
use crate::process::context::Context;
use crate::process::process::ProcessStatus::Waiting;
use crate::utility::recycle_counter::RecycleCounter;

//todo: warp unsafe mutable static with lock
static mut PID_ALLOCATOR: RecycleCounter = RecycleCounter::new(usize::MAX - 1);

pub enum ProcessStatus {
    Running,
    Waiting,
    Ready,
    Dead
}

pub(crate) struct Process {
    pub pid: usize,
    pub exit_code:i32,
    pub context: Context,
    pub status: ProcessStatus,
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
    pub parent: Option<Weak<Process>>,
    pub children: Vec<Arc<Process>>,
}

impl Process {
    fn area_loading(&mut self, area: MapArea, data: Option<&[u8]>) {
        if data.is_none() { return; }
        let mut ptr: usize = 0;
        let mut vpn = area.start;
        while ptr < data.len() {
            let src = data[ptr..min(ptr + PAGE_SIZE, data.len())];
            let des = read_frame(area.frame_mapping.get(&vpn).unwrap() as PhysPageNum)[..src.len()];
            dst.copy_from_slice(src);
            ptr += PAGE_SIZE;
            vpn += 1;
        }
    }
    
    fn frame_recycle(&mut self){
        for mut area in self.areas{
            area.recycle();
        }
    }

    pub fn load_elf(elf_data: &[u8]) -> Process {
        let pg_root = frame_alloc().unwrap();
        let mut page_table = PageTable::new(pg_root);
        page_table.load_trampoline();
        let pid: usize;
        unsafe {
            pid = PID_ALLOCATOR.alloc().unwrap();
        }
        let mut process = Process {
            pid,
            exit_code: 0,
            context: Context::new(),
            status: Waiting,
            page_table,
            areas: vec![],
            parent: None,
            children: vec![],
        };
        KERNEL_SPACE.exclusive_access().kernel_stack_apply(pid);
        let elf = xmas_elf::ElfFile::new(elf_data).unwrap();
        let elf_header = elf.header;
        let magic = elf_header.pt1.magic;
        assert_eq!(magic, [0x7f, 0x45, 0x4c, 0x46], "invalid elf!");
        let mut max_end_vpn: VirPageNum = 0;
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
                    map_perm != MAP_PERM_X;
                }
                let map_area = MapArea::new(start_va, end_va, Framed, map_perm);
                max_end_vpn = map_area.end;
                process.page_table.area_mapping(map_area);
                process.area_loading(map_area, Some(&elf.input[ph.offset() as usize..(ph.offset() + ph.file_size()) as usize]));
                process.areas.push(map_area);
            }
        }
        let user_stack_bottom = page_num_to_addr(max_end_vpn) + PAGE_SIZE;
        let user_stack_top = user_stack_bottom + USER_STACK_SIZE;
        let user_stack_area = MapArea::new(user_stack_bottom, user_stack_top, Framed, MAP_PERM_U | MAP_PERM_R | MAP_PERM_W);
        process.page_table.area_mapping(user_stack_area);
        process.areas.push(user_stack_area);
        process
    }

    pub fn clone(obj: &Self) -> Self {
        let mut pid: usize;
        unsafe {
            pid = PID_ALLOCATOR.alloc().unwrap();
        }
        let pg_root = frame_alloc().unwrap();
        let mut this = Process {
            pid,
            exit_code: 0,
            context: (Context::new()),
            status: ProcessStatus::Waiting,
            page_table: (PageTable::new(pg_root)),
            areas: vec![],
            parent: None,
            children: vec![],
        };
        this.page_table.load_trampoline();
        for area in obj.areas.iter(){
            let cur_area=area.clone();
            this.page_table.area_mapping(cur_area);
            this.areas.push(cur_area);
            for vpn in area.start..=area.end{
                let src=obj.page_table.find_pte(vpn).unwrap().ppn();
                let dst = this.page_table.find_pte(vpn).unwrap().ppn();
                read_frame(dst).copy_from_slice(read_frame(src));
            }
        }
        this
    }

    pub fn activate(&self){
        let satp=self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }
}