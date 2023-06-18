use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use crate::mm::{ceiling, floor, PhysPageNum, VirAddr, VirPageNum};
use crate::mm::frame_allocator::frame_dealloc;

#[derive(Copy, Clone)]
pub enum MapType{
    Identical,
    Framed
}

//identical to PTE flags
pub const MAP_PERM_R:usize=1<<1;
pub const MAP_PERM_W:usize=1<<2;
pub const MAP_PERM_X:usize=1<<3;
pub const MAP_PERM_U:usize=1<<4;


#[derive(Copy, Clone)]
pub struct MapArea {
    pub start: VirPageNum,
    pub end: VirPageNum,
    //actual mem [start, end]
    pub frame_mapping: BTreeMap<VirPageNum,PhysPageNum>,
    pub map_type:MapType,
    pub map_perm:usize
}

impl MapArea {
    pub fn new (start_va:VirAddr,end_va:VirAddr,map_type:MapType,map_perm:usize)->Self{
        MapArea {
            start:floor(start_va) ,
            end:ceiling(end_va),
            frame_mapping: BTreeMap::new(),
            map_type,
            map_perm
        }
    }

    pub fn recycle(&mut self){
        for (_,ppn )in self.frame_mapping.iter(){
            frame_dealloc(ppn as PhysPageNum);
        }
    }
}