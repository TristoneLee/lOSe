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
            if self.cnt==self.size {
                None
            }else {
                self.cnt=self.cnt+1;
                Some(self.cnt-1)
            }
        }else{
            self.recycle.pop()
        }
    }

    pub fn dealloc(& mut self, idx:usize){
        self.recycle.push(idx)
    }
}