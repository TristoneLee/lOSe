use alloc::vec;
use alloc::vec::Vec;
use core::borrow::{Borrow, BorrowMut};
use core::ops::Deref;
use lazy_static::lazy_static;
use crate::mm::{ceiling, ekernel, floor, MEMORY_END, PhysPageNum, read_bytes_array, to_ppn};
use crate::{print, println};
use crate::utility::recycle_counter::RecycleCounter;
use crate::sync::cell::Mutex;

pub struct FrameAllocator{
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
    pub static ref FRAME_ALLOCATOR:Mutex<FrameAllocator>= Mutex::new(FrameAllocator::new());
}

pub fn frame_allocator_init(){
    extern "C"{
        pub fn ekernel();
    }
    let begin=ceiling(ekernel as usize);
    let end = floor(MEMORY_END);
    FRAME_ALLOCATOR.lock().init(begin,end)
}

pub fn frame_alloc()->Option<PhysPageNum>{
    let result=FRAME_ALLOCATOR.lock().alloc();
    if let Some (ppn)=result{
        let bytes_array = read_bytes_array(ppn);
        for i in bytes_array {
            *i = 0;
        }
    }
    result
}

pub fn frame_dealloc(recycle: PhysPageNum){
    FRAME_ALLOCATOR.lock().dealloc(recycle)
}