use alloc::vec;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::Deref;
use lazy_static::lazy_static;
use crate::mm::{ceiling, ekernel, floor, MEMORY_END, PhysPageNum, to_ppn};
use crate::{print, println};
use crate::utility::recycle_counter::RecycleCounter;
use crate::sync::cell::UPSafeCell;

struct FrameAllocator{
    current: PhysPageNum,
    end: PhysPageNum,
    recycled: Vec<PhysPageNum>,
}

impl FrameAllocator{
    pub fn new()->Self{
        FrameAllocator{
            current: 0,
            end: 0,
            recycled: vec![],
        }
    }

    pub fn init(&mut self,begin:PhysPageNum,end:PhysPageNum){
        self.current=begin;
        self.end=end;
    }

    pub fn alloc(&mut self)->Option<PhysPageNum>{
        if !self.recycled.is_empty(){
            self.recycled.pop()
        }else if self.current==self.end{
            println!("[Warning]: Frame allocator running out");
            None
        }else {
            self.current+=1;
            Some(self.current-1)
        }
    }

    pub fn dealloc(&mut self,recycle:PhysPageNum){
        self.recycled.push(recycle)
    }

}


lazy_static! {
    pub static ref FRAME_ALLOCATOR:UPSafeCell<FrameAllocator>=FrameAllocator::new();
}

pub fn frame_allocator_init(){
    extern "C"{
        pub fn ekernel();
    }
    let begin=ceiling(ekernel as usize);
    let end = floor(MEMORY_END);
    FRAME_ALLOCATOR.exclusive_access().init(begin,end)
}

pub fn frame_alloc()->Option<PhysPageNum>{
    FRAME_ALLOCATOR.exclusive_access().alloc()
}

pub fn frame_dealloc(recycle: PhysPageNum){
    FRAME_ALLOCATOR.exclusive_access().dealloc(recycle)
}