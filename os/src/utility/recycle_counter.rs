use alloc::vec;
use alloc::vec::Vec;

pub struct RecycleCounter{
    recycle:Vec<usize>,
    cnt:usize,
    size:usize
}

impl RecycleCounter {
    pub fn new(size:usize)->Self{
        RecycleCounter{
            recycle: vec![],
            cnt: 0,
            size
        }
    }

    pub fn alloc(&mut self)->Option<usize>{
        if self.recycle.is_empty(){
            if cnt==size {
                None
            }else {
                cnt=cnt+1;
                Option(cnt-1)
            }
        }else{
            self.recycle.pop()
        }
    }

    pub fn dealloc(& mut self, idx:usize){
        self.recycle.push(idx)
    }
}