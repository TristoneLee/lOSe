use crate::mm::PhysPageNum;
use crate::utility::recycle_counter::RecycleCounter;

struct FrameAllocator{
    counter:RecycleCounter,
    base:usize
}

pub fn frame_alloc()->Option<PhysPageNum>{

}

pub fn frame_dealloc(frame: PhysPageNum){

}