use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;
use crate::mm::{get_offset, get_vir_indexes, PA_WIDTH, page_num_to_addr, PAGE_WIDTH, PhyAddr, PhysPageNum, PPN_WIDTH, strampoline, to_ppn, to_va, to_vpn, TRAMPOLINE, VirAddr, VirPageNum};
use crate::mm::frame_allocator::frame_alloc;
use bitflags::*;
use crate::mm::map_area::{MapArea, MapType};

// enum PTEFlags {
//     V,
//     //Valid
//     R,
//     //Readable
//     W,
//     //Writable
//     X,
//     //Executable
//     U,
//     //User
//     G,
//     //Global
//     A,
//     //Accessed
//     D,  //Dirty
// }

const PTE_FLAG_V: usize = 1;
const PTE_FLAG_R: usize = 1 << 1;
const PTE_FLAG_W: usize = 1 << 2;
const PTE_FLAG_X: usize = 1 << 3;
const PTE_FLAG_U: usize = 1 << 4;
const PTE_FLAG_G: usize = 1 << 5;
const PTE_FLAG_A: usize = 1 << 6;
const PTE_FLAG_D: usize = 1 << 7;
const PTE_NO_FLAG: usize = 0;

pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flag: usize) -> Self {
        PageTableEntry((ppn << 10) & flag)
    }

    pub fn empty() -> Self {
        PageTableEntry(0)
    }

    pub fn ppn(&self) -> PhysPageNum {
        self.0 >> 10 & (1 << PPN_WIDTH - 1)
    }

    pub fn is_valid(&self) -> bool {
        (self.0 & 1) == 1
    }

    pub fn is_readable(&self) -> bool {
        (self.0 >> 1) & 1 == 1
    }

    pub fn is_writable(&self) -> bool {
        (self.0 >> 2) & 1 == 1
    }

    pub fn is_executable(&self) -> bool {
        (self.0 >> 3) & 1 == 1
    }
}

pub struct PageTable {
    root: PhysPageNum,
    frames: Vec<PhysPageNum>,
}

impl PageTable {
    pub fn new(root: PhysPageNum) -> Self {
        let frame = frame_alloc().unwrap();
        PageTable {
            root: frame,
            frames: vec![frame],
        }
    }

    pub fn area_mapping(&mut self, mut area: MapArea) {
        for vpn in area.start..=area.end {
            match area.map_type {
                MapType::Identical => {
                    let ppn: PhysPageNum = vpn;
                    self.map(vpn, ppn, area.map_perm);
                    area.frame_mapping.insert(vpn, frame)
                }
                MapType::Framed => {
                    let ppn = frame_alloc().unwrap();
                    self.map(vpn, frame, area.map_perm);
                    area.frame_mapping.insert(vpn, frame)
                }
            }
        }
    }

    pub fn token(&self)->usize{
        8usize << 60 | self.root
    }

    pub fn load_trampoline(&mut self) {
        unsafe {
            self.map(to_vpn(TRAMPOLINE), to_ppn(strampoline as usize),PTE_FLAG_R|PTE_FLAG_X)
        }
    }

    pub fn find_pte_create(&mut self, vpn: VirPageNum) -> Option<&mut PageTableEntry> {
        let indexes = get_vir_indexes(vpn);
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, index) in indexes.iter().enumerate() {
            let mut pte_ptr: *mut usize = ((ppn << PAGE_WIDTH) + index * size_of::<usize>()) as *mut usize;
            let mut pte = PageTableEntry::empty();
            unsafe {
                pte.0 = *pte_ptr;
            }
            if i == 2 {
                result = Some(&mut pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                unsafe {
                    *pte_ptr = PageTableEntry::new(frame, PTE_FLAG_V).0;
                }
                self.frames.push(frame);
            }
            ppn = pte.ppn();
        }
        result
    }

    pub fn find_pte(&self, vpn: VirPageNum) -> Option<&mut PageTableEntry> {
        let indexes = get_vir_indexes(vpn);
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, index) in indexes.iter().enumerate() {
            let mut pte_ptr: *mut usize = ((ppn << PAGE_WIDTH) + index * size_of::<usize>()) as *mut usize;
            let mut pte = PageTableEntry::empty();
            unsafe {
                pte.0 = *pte_ptr;
            }
            if i == 2 {
                result = Some(&mut pte);
                break;
            }
            if !pte.is_valid() {
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    pub fn map(&mut self, vpn: VirPageNum, ppn: PhysPageNum, flag: usize) {
        let pte = self.find_pte_create(vpn).unwrap();
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        *pte = PageTableEntry::new(ppn, flag | PTE_FLAG_V);
    }

    pub fn unmap(&mut self, vpn: VirPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn translate_va(& self,va:VirAddr)->Option<PhyAddr>{
        if let Some(pte)=self.find_pte(page_num_to_addr(va)){
            Some(page_num_to_addr(pte.ppn())+get_offset(va));
        }
        None
    }
}
