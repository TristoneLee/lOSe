use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::lazy_static;
use riscv::register::satp;
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::map_area::{MAP_PERM_R, MAP_PERM_W, MAP_PERM_X, MapType, MapArea};
use crate::mm::pagetable::PageTable;
use crate::mm::{ebss, edata, ekernel, erodata, etext, KERNEL_STACK_SIZE, MEMORY_END, PAGE_SIZE, sbss_with_stack, sdata, srodata, stext, TRAMPOLINE};
use crate::mm::map_area::MapType::{Framed, Identical};
use crate::sync::cell::UPSafeCell;

struct KernelSpace {
    page_table: PageTable,
    areas: Vec<MapArea>,
}

lazy_static! {
    pub static ref KERNEL_SPACE:UPSafeCell<KernelSpace>=KernelSpace::new();
}

impl KernelSpace {
    pub fn new() -> Self {
        let pg_root = frame_alloc().unwrap();
        let mut result = KernelSpace {
            page_table: PageTable::new(pg_root),
            areas: vec![],
        };
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
    }

    pub fn kernel_token(&self) -> usize {
        self.page_table.token()
    }

    pub fn init(&mut self) {
        self.page_table.load_trampoline();
        self.areas.push(MapArea::new(
            stext as usize,
            etext as usize,
            Identical,
            MAP_PERM_R | MAP_PERM_X,
        ));
        self.areas.push(
            MapArea::new(
                srodata as usize,
                erodata as usize,
                Identical,
                MAP_PERM_R,
            )
        );
        self.areas.push(
            MapArea::new(
                sdata as usize,
                edata as usize,
                Identical,
                MAP_PERM_R | MAP_PERM_W,
            )
        );
        self.areas.push(
            MapArea::new(
                sbss_with_stack as usize,
                ebss as usize,
                Identical,
                MAP_PERM_R | MAP_PERM_W,
            )
        );
        self.areas.push(
            MapArea::new(
                ekernel as usize,
                MEMORY_END,
                Identical,
                MAP_PERM_R | MAP_PERM_W,
            )
        );
        //todo MMIO mapping
        self.activate();
    }

    pub fn kernel_stack_apply(&mut self, pid: usize) -> usize {
        let top = TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE);
        let bottom = top - KERNEL_STACK_SIZE;
        self.page_table.area_mapping(MapArea::new(bottom, top, Framed, MAP_PERM_R | MAP_PERM_W));
        top
    }

    pub fn kernel_stack_recycle(&mut self, pid: usize) {
        //todo
    }
}

pub fn kernel_stack_top(pid: usize) -> usize {
    TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE)
}

pub fn kernel_stack_bottom(pid: usize) -> usize {
    TRAMPOLINE - pid * (KERNEL_STACK_SIZE + PAGE_SIZE) - KERNEL_STACK_SIZE
}