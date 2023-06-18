use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::mem::size_of;
use crate::mm::{addr_to_page_num, floor, get_mut, get_offset, get_vir_indexes, page_num_to_addr, PAGE_WIDTH, PhyAddr, PhysPageNum, PPN_WIDTH, read_frame, read_pte_array, strampoline, to_ppn, to_va, to_vpn, TRAMPOLINE, VirAddr, VirPageNum};
use crate::mm::frame_allocator::frame_alloc;
use bitflags::*;
use crate::io::print;
use crate::mm::map_area::{MapArea, MapType};
use crate::println;

const PTE_FLAG_V: usize = 1;
const PTE_FLAG_R: usize = 1 << 1;
const PTE_FLAG_W: usize = 1 << 2;
const PTE_FLAG_X: usize = 1 << 3;
const PTE_FLAG_U: usize = 1 << 4;
const PTE_FLAG_G: usize = 1 << 5;
const PTE_FLAG_A: usize = 1 << 6;
const PTE_FLAG_D: usize = 1 << 7;
const PTE_NO_FLAG: usize = 0;

#[derive(Clone,Copy)]
pub struct PageTableEntry(usize);

impl PageTableEntry {
    pub fn new(ppn: PhysPageNum, flag: usize) -> Self {
        // println!("create pte {:#b}",(ppn << 10) | flag);
        PageTableEntry((ppn << 10) | flag)
    }

    pub fn empty() -> Self {
        PageTableEntry(0)
    }

    pub fn ppn(&self) -> PhysPageNum {
        (self.0 >> 10 )& ((1 << PPN_WIDTH) - 1)
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
        PageTable {
            root,
            frames: vec![root],
        }
    }

    pub fn from_token(satp: usize) -> Self {
        Self {
            root: satp & ((1usize << PPN_WIDTH) - 1),
            frames: Vec::new(),
        }
    }

    pub fn area_mapping(&mut self, area: &mut MapArea) {
        for vpn in area.start..area.end {
            match area.map_type {
                MapType::Identical => {
                    let frame: PhysPageNum = vpn;
                    self.map(vpn, frame, area.map_perm);
                    area.frame_mapping.insert(vpn, frame);
                }
                MapType::Framed => {
                    let frame = frame_alloc().unwrap();
                    self.map(vpn, frame, area.map_perm);
                    area.frame_mapping.insert(vpn, frame);
                }
            }
        }
    }

    pub fn token(&self)->usize{
        8usize << 60 | self.root
    }

    pub fn root_ppn(&self)->usize{
        self.root
    }

    pub fn load_trampoline(&mut self) {
        println!("Loading trampoline at VA {:#x} for page table {:#x}",strampoline as usize,self.root);
        unsafe {
            self.map(addr_to_page_num(TRAMPOLINE), addr_to_page_num(strampoline as usize),PTE_FLAG_R|PTE_FLAG_X);
        }
        println!("Trampoline loaded");
    }

    pub fn find_pte_create(&mut self, vpn: VirPageNum) -> Option<&mut PageTableEntry> {
        let indexes = get_vir_indexes(vpn);
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, index) in indexes.iter().enumerate() {
            // println!("iter {}, index {}, at ppn {:#b}",i,index,ppn);
            let pte = &mut read_pte_array(ppn)[*index];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                let frame = frame_alloc().unwrap();
                *pte = PageTableEntry::new(frame, PTE_FLAG_V);
                // println!("{:#b}",pte.0);
                self.frames.push(frame);
            }
            // println!("{:#b}",pte.ppn());
            ppn = pte.ppn();
        }
        result
    }

    pub fn find_pte(&self, vpn: VirPageNum) -> Option<&mut PageTableEntry> {
        let indexes = get_vir_indexes(vpn);
        let mut ppn = self.root;
        let mut result: Option<&mut PageTableEntry> = None;
        for (i, index) in indexes.iter().enumerate() {
            let pte = &mut read_pte_array(ppn)[*index];
            if i == 2 {
                result = Some(pte);
                break;
            }
            if !pte.is_valid() {
                println!("Fail to find va{:#x} in pagetable {:#x}",page_num_to_addr(vpn),self.root);
                return None;
            }
            ppn = pte.ppn();
        }
        result
    }

    pub fn map(&mut self, vpn: VirPageNum, ppn: PhysPageNum, flag: usize) {
        let pte = self.find_pte_create(vpn).unwrap();
        // println!("vpn {} mapped to ppn{}",vpn,ppn);
        assert!(!pte.is_valid(), "vpn {:?} is mapped before mapping", vpn);
        // println!("pagetable mapping flag {:#b}",flag|PTE_FLAG_V);
        *pte = PageTableEntry::new(ppn, flag | PTE_FLAG_V);
    }

    pub fn unmap(&mut self, vpn: VirPageNum) {
        let pte = self.find_pte(vpn).unwrap();
        assert!(pte.is_valid(), "vpn {:?} is invalid before unmapping", vpn);
        *pte = PageTableEntry::empty();
    }

    pub fn translate_va(& self,va:VirAddr)->Option<PhyAddr>{
        if let Some(pte)=self.find_pte(addr_to_page_num(va)){
            return Some(page_num_to_addr(pte.ppn())+get_offset(va));
        }
        None
    }

    pub fn translated_str(&self,ptr: *const u8) -> String {
        let mut string = String::new();
        let mut va = to_va(ptr as usize) ;
        loop {
            let ch: u8 = *(get_mut(self.translate_va(va).unwrap()));
            if ch == 0 {
                break;
            } else {
                string.push(ch as char);
                va += 1;
            }
        }
        string
    }

    pub fn translated_byte_buffer(&self, ptr: *const u8, len: usize) -> Vec<&'static mut [u8]> {
        let mut start = ptr as usize;
        let end = start + len;
        let mut v = Vec::new();
        let mut iter =0;
        while start < end {
            iter+=1;
            let start_va = to_va(start);
            let mut vpn: VirPageNum = floor(start_va);
            let ppn = self.find_pte(vpn).unwrap().ppn();
            vpn+=1;
            let mut end_va: VirAddr = page_num_to_addr(vpn);
            end_va = end_va.min(to_va(end));
            if get_offset(end_va) == 0 {
                v.push(&mut read_frame(ppn)[get_offset(start_va)..]);
            } else {
                v.push(&mut read_frame(ppn)[get_offset(start_va)..get_offset(end_va)]);
            }
            start = end_va;
        }
        v
    }
}
