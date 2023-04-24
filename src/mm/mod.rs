pub mod buddy_allocator;
pub mod pagetable;
pub mod frame_allocator;
pub mod map_area;
pub mod kernel_space;

pub type PhysPageNum = usize;
pub type PhyAddr = usize;
pub type VirPageNum = usize;
pub type VirAddr = usize;

pub const PA_WIDTH: usize = 56;
pub const VA_WIDTH: usize = 39;
pub const PPN_WIDTH: usize = 44;
pub const VPN_WIDTH: usize = 27;
pub const PA_OFFSET: usize = 11;
pub const VA_OFFSET: usize = 9;
pub const PAGE_SIZE: usize = 1 << 12;
pub const PAGE_WIDTH: usize = 12;
pub const MAX_VA: usize = usize::MAX - 1;
pub const TRAMPOLINE: usize = MAX_VA - PAGE_SIZE;
pub const MEMORY_END: usize = 0x81000000;
pub const USER_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_STACK_SIZE: usize = 4096 * 2;
pub const KERNEL_HEAP_SIZE: usize = 0x20_0000;

extern "C" {
    pub fn stext();
    pub fn etext();
    pub fn srodata();
    pub fn erodata();
    pub fn sdata();
    pub fn edata();
    pub fn sbss_with_stack();
    pub fn ebss();
    pub fn ekernel();
    pub fn strampoline();
}

pub fn to_pa(v: usize) -> PhyAddr {
    v & (1 << PA_WIDTH - 1)
}

pub fn to_va(v: usize) -> VirAddr {
    v & (1 << VA_WIDTH - 1)
}

pub fn to_ppn(v: usize) -> PhysPageNum {
    v & (1 << PPN_WIDTH - 1)
}

pub fn to_vpn(v: usize) -> VirPageNum {
    v & (1 << VA_WIDTH - 1)
}

pub fn addr_to_page_num(v: usize) -> usize {
    v >> PAGE_WIDTH
}

pub fn page_num_to_addr(v: usize) -> usize { v << PAGE_WIDTH }

pub fn floor(v: usize) -> usize {
    v / PAGE_SIZE
}

pub fn ceiling(v: usize) -> usize {
    (v - 1) / PAGE_SIZE + 1
}

pub fn get_offset(v: usize) -> usize {
    v & (PAGE_SIZE - 1)
}

pub fn read_frame(ppn: PhysPageNum) -> &'static mut [u8] {
    let pa: PhyAddr = page_num_to_addr(ppn);
    unsafe { core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096) }
}

pub fn get_vir_indexes(v: VirPageNum) -> [usize; 3] {
    [v & ((1 << VA_OFFSET) - 1),
        v >> VA_OFFSET & ((1 << VA_OFFSET) - 1),
        v >> (2 * VA_OFFSET) & ((1 << VA_OFFSET) - 1)]
}

pub fn get_phys_indexes(v: PhysPageNum) -> [usize; 3] {
    [v & ((1 << PA_OFFSET) - 1),
        v >> PA_OFFSET & ((1 << PA_OFFSET) - 1),
        v >> (2 * PA_OFFSET) & ((1 << PA_OFFSET) - 1)]
}

// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
// struct PhysAddr(usize);
//
// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
// struct PhysPageNum(usize);
//
// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
// struct VirAddr(usize);
//
// #[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
// struct VirPageNum(usize);
//
// impl From<usize> for PhysAddr {
//     fn from(value: usize) -> Self {
//         Self(value & ((1 << PA_WIDTH) - 1))
//     }
// }
//
// impl From<usize> for VirAddr {
//     fn from(value: usize) -> Self {
//         Self(value & ((1 << VA_WIDTH) - 1))
//     }
// }
//
// impl From<usize> for PhysPageNum {
//     fn from(value: usize) -> Self {
//         Self(value & ((1 << PPN_WIDTH) - 1))
//     }
// }
//
// impl From<usize> for VirPageNum {
//     fn from(value: usize) -> Self {
//         Self(value & ((1 << VPN_WIDTH) - 1))
//     }
// }
