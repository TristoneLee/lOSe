use alloc::vec;
use alloc::vec::Vec;
use core::arch::asm;
use lazy_static::lazy_static;
use riscv::register::satp;
use crate::io::print;
use crate::io::uart::UART0;
use crate::mm::frame_allocator::frame_alloc;
use crate::mm::map_area::{MAP_PERM_R, MAP_PERM_W, MAP_PERM_X, MapType, MapArea};
use crate::mm::pagetable::PageTable;
use crate::mm::{ebss, edata, ekernel, erodata, etext, KERNEL_STACK_SIZE, MEMORY_END, PAGE_SIZE, sbss_with_stack, sdata, srodata, stext, TRAMPOLINE};
use crate::mm::map_area::MapType::{Framed, Identical};
use crate::println;
use crate::sync::cell::Mutex;
use crate::syscall::VIRT_TEST;
use crate::utility::timer::CLINT;

pub struct KernelSpace {
    pub page_table: PageTable,
    pub areas: Vec<MapArea>,
}

lazy_static! {
    pub static ref KERNEL_SPACE:Mutex<KernelSpace>= Mutex::new(KernelSpace::new());
}

impl KernelSpace {
    pub fn new() -> Self {
        let pg_root = frame_alloc().unwrap();
        let mut result = KernelSpace {
            page_table: PageTable::new(pg_root),
            areas: vec![],
        };
        result
    }

    pub fn activate(&self) {
        let satp = self.page_table.token();
        unsafe {
            satp::write(satp);
            asm!("sfence.vma");
        }
        println!("Kernel page table online");
    }

    pub fn kernel_token(&self) -> usize {
        self.page_table.token()
    }

    pub fn init(&mut self) {
        extern "C" {
            fn stext();
            fn etext();
            fn srodata();
            fn erodata();
            fn sdata();
            fn edata();
            fn sbss_with_stack();
            fn ebss();
            fn ekernel();
        }
        println!("Kernel space load trampoline");
        self.page_table.load_trampoline();
        println!(".text [{:#x}, {:#x})", stext as usize, etext as usize);
        println!(".rodata [{:#x}, {:#x})", srodata as usize, erodata as usize);
        println!(".data [{:#x}, {:#x})", sdata as usize, edata as usize);
        println!(".bss [{:#x}, {:#x})", sbss_with_stack as usize, ebss as usize);
        println!("Kernel space load text");
        let mut text_area = MapArea::new(
            stext as usize,
            etext as usize,
            Identical,
            MAP_PERM_R | MAP_PERM_X,
        );
        self.page_table.area_mapping(&mut text_area);
        self.areas.push(text_area);
        println!("Kernel space load rodata");
        let mut rodata_area = MapArea::new(
            srodata as usize,
            erodata as usize,
            Identical,
            MAP_PERM_R,
        );
        self.page_table.area_mapping(&mut rodata_area);
        self.areas.push(rodata_area);
        println!("Kernel space load data");
        let mut data_area = MapArea::new(
            sdata as usize,
            edata as usize,
            Identical,
            MAP_PERM_R | MAP_PERM_W,
        );
        self.page_table.area_mapping(&mut data_area);
        self.areas.push(
            data_area
        );
        println!("Kernel space load bss");
        let mut bss_area = MapArea::new(
            sbss_with_stack as usize,
            ebss as usize,
            Identical,
            MAP_PERM_R | MAP_PERM_W,
        );
        self.page_table.area_mapping(&mut bss_area);
        self.areas.push(bss_area);
        println!("Kernel space load physical memory");
        let mut kernel_heap_area = MapArea::new(
            ekernel as usize,
            MEMORY_END,
            Identical,
            MAP_PERM_R | MAP_PERM_W,
        );
        self.page_table.area_mapping(&mut kernel_heap_area);
        self.areas.push(kernel_heap_area);
        let mut uart_area = MapArea::new(
            UART0 as usize,
            UART0 as usize + 10,
            Identical,
            MAP_PERM_R | MAP_PERM_W,
        );
        self.page_table.area_mapping(&mut uart_area);
        self.areas.push(uart_area);
        let mut virt_area=MapArea::new(
            VIRT_TEST as usize,
            VIRT_TEST as usize +100,
            Identical,
            MAP_PERM_R|MAP_PERM_W
        );
        self.page_table.area_mapping(&mut virt_area);
        self.areas.push(virt_area);
        let mut clint_area=MapArea::new(
            CLINT as usize,
            CLINT as usize +0x10000,
            Identical,
            MAP_PERM_R|MAP_PERM_W
        );
        self.page_table.area_mapping(&mut clint_area);
        self.areas.push(clint_area);
        self.activate();
    }

    pub fn kernel_stack_apply(&mut self, pid: usize) -> usize {
        let top = kernel_stack_top(pid);
        let bottom = top - KERNEL_STACK_SIZE;
        println!("Kernel stack apply from {:#x} to {:#x}",bottom, top);
        self.page_table.area_mapping(&mut MapArea::new(bottom, top, Framed, MAP_PERM_R | MAP_PERM_W));
        top
    }

}

pub fn kernel_stack_top(pid: usize) -> usize {
    TRAMPOLINE - (pid+1) * (KERNEL_STACK_SIZE + PAGE_SIZE)
}

pub fn kernel_stack_bottom(pid: usize) -> usize {
    TRAMPOLINE - (pid+1) * (KERNEL_STACK_SIZE + PAGE_SIZE) - KERNEL_STACK_SIZE
}