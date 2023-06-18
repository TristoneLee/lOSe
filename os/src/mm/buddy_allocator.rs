use core::alloc::{GlobalAlloc, Layout};
use core::cmp::{max, min};
use core::mem::size_of;
use core::ops::Deref;
use core::ptr::{NonNull, null, null_mut};
use buddy_system_allocator::LockedHeap;
use crate::mm::KERNEL_HEAP_SIZE;
use crate::println;
use crate::sync::cell::Mutex;

#[global_allocator]
static HEAP_ALLOCATOR: BuddyAllocator = BuddyAllocator::new();
#[link_section = ".data.heap"]
static mut KERNEL_HEAP_SPACE: [u8; KERNEL_HEAP_SIZE] = [0; KERNEL_HEAP_SIZE];

// #[alloc_error_handler]
// pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
//     panic!("Heap allocation error, layout = {:?}", layout);
// }

pub fn heap_init() {
    unsafe {
        HEAP_ALLOCATOR.lock()
            .init(KERNEL_HEAP_SPACE.as_ptr() as usize,  KERNEL_HEAP_SIZE);
    }
}

#[derive(Copy, Clone)]
struct MMListNode {
    prev: *mut usize,
    cur: *mut usize,
}

impl MMListNode {
    pub fn iter(&mut self) -> bool {
        return if self.cur.is_null() {
            false
        } else {
            self.prev = self.cur;
            self.cur = unsafe { *self.cur as *mut usize };
            true
        };
    }

    pub fn cur(&self) -> *mut usize {
        self.cur
    }

    pub fn pop(&mut self) {
        unsafe {
            if !self.prev.is_null() {
                *self.prev = *self.cur;
            }
            self.cur = *self.cur() as *mut usize;
        }
    }

    pub fn if_null(&self) -> bool {
        return self.cur.is_null();
    }
}


#[derive(Copy, Clone)]
struct MMLinkedList {
    head: *mut usize,
}

unsafe impl Send for MMLinkedList {}

impl MMLinkedList {
    pub const fn new() -> Self {
        MMLinkedList {
            head: null_mut(),
        }
    }

    pub fn head_node(&mut self) -> MMListNode {
        MMListNode {
            prev: null_mut(),
            cur: self.head,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    pub unsafe fn push(&mut self, node: *mut usize) {
        *node = self.head as usize;
        self.head = node;
    }

    pub unsafe fn pop(&mut self) -> Option<*mut usize> {
        if self.head.is_null() {
            None
        } else {
            let tmp = self.head;
            self.head = *tmp as *mut usize;
            Some(tmp)
        }
    }
}

pub fn prev_power_of_two(num: usize) -> usize {
    1 << (8 * (size_of::<usize>()) - num.leading_zeros() as usize - 1)
}

pub fn lowbit(v: usize) -> usize {
    v & (!v + 1)
}

pub struct BuddyInner {
    free_list: [MMLinkedList; 32],
    user: usize,
    size: usize,
    occupied: usize,
}

impl BuddyInner {
    pub const fn new() -> Self {
        BuddyInner {
            free_list: [MMLinkedList::new(); 32],
            user: 0,
            size: 0,
            occupied: 0,
        }
    }

    pub unsafe fn add_to_head(&mut self, start: usize, end: usize) {
        let start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        let end = end & (!size_of::<usize>() + 1);
        let mut current_start = start;
        while current_start + size_of::<usize>() <= end {
            let to_alloc = min(lowbit(current_start), prev_power_of_two(end - current_start));
            self.free_list[to_alloc.trailing_zeros() as usize].push(current_start as *mut usize);
            self.size += to_alloc;
            current_start += to_alloc;
        }
    }

    pub unsafe fn init(&mut self, start :usize, size:usize){
        self.add_to_head(start,start+size);
    }
}

impl BuddyInner {
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()));
        let power = size.trailing_zeros() as usize;
        for i in power..self.free_list.len() {
            if !self.free_list[i].is_empty() {
                for j in (power + 1..i + 1).rev() {
                    let new_block = self.free_list[j].pop().unwrap();
                    self.free_list[j - 1].push(new_block);
                    self.free_list[j - 1].push((new_block as usize + (1 << (j - 1))) as *mut usize);
                }
                let result = self.free_list[power].pop().unwrap();
                self.user += layout.size();
                self.occupied += size;
                return result as *mut u8;
            }
        }
        panic!("Buddy allocator run out");
    }

    unsafe fn dealloc(&mut self, ptr: *mut usize, layout: Layout) {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::<usize>()),
        );
        let power = size.trailing_zeros() as usize;
        self.free_list[power].push(ptr );
        let mut current_ptr = ptr as usize;
        let mut current_power = power;
        while current_power < self.free_list.len() {
            let buddy = current_ptr ^ (1 << current_power);
            let mut flag = false;
            let mut node = self.free_list[current_power].head_node();
            loop {
                if node.cur() as usize == buddy {
                    node.pop();
                    flag = true;
                    break;
                }
                if !node.iter() {
                    break;
                }
            }
            if flag {
                self.free_list[current_power].pop();
                current_ptr = min(current_ptr, buddy);
                current_power += 1;
                self.free_list[current_power].push(current_ptr as *mut usize);
            } else {
                break;
            }
        }
        self.user -= layout.size();
        self.occupied -= size;
    }
}

struct BuddyAllocator(Mutex<BuddyInner>);

impl BuddyAllocator {
    pub const fn new() -> Self {
        Self(Mutex::new(BuddyInner::new()))
    }
}

impl Deref for BuddyAllocator {
    type Target = Mutex<BuddyInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl GlobalAlloc for BuddyAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.0.lock().alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.0.lock().dealloc(ptr as *mut usize, layout)
    }
}
