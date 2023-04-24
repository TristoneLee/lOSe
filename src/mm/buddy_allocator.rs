use alloc::collections::LinkedList;
use core::alloc::Layout;
use core::cmp::{max, min};
use core::mem::size_of;
use core::ptr::{NonNull, null_mut};

#[derive(Copy, Clone)]
struct MMLinkedList {
    head: *mut usize,
    cnt: usize,
}

impl MMLinkedList {
    pub fn new(&mut self) -> Self {
        MMLinkedList {
            head: null_mut(),
            cnt: 0,
        }
    }

    pub fn is_empty(& self) -> bool {
        self.cnt == 0
    }
    pub unsafe fn push(&mut self, node: *mut usize) {
        *node = self.head as usize;
        self.head = node;
        self.cnt += 1;
    }

    pub unsafe fn pop(&mut self) -> Option<* usize> {
        if self.cnt == 0 {
            None
        } else {
            let tmp = self.head;
            self.head = *head;
            self.cnt -= 1;
            Some(tmp)
        }
    }
}

pub fn prev_power_of_two(v: usize) -> usize {
    let mut result: usize = 0;
    let mut tmp = v;
    while tmp != 0 {
        tmp >>= 1;
        result += 1;
    }
    result - 1
}

pub fn next_power_of_two(v: usize) -> usize {
    v.next_power_of_two()
}

pub fn lowbit(v: usize) -> usize {
    v & (!v + 1)
}

// A frame allocator based on buddy algorithm
struct BuddyAllocator {
    free_list: [MMLinkedList; 32],

    base_frame: usize,
    size: usize,
    occupied: usize,
}

impl BuddyAllocator {
    pub fn new() -> Self {
        BuddyAllocator {
            free_list: Default::default(),
            base_frame: 0,
            size: 0,
            occupied: 0,
        }
    }

    pub unsafe fn add_to_head(&mut self, mut start: usize, mut end: usize) {
        start = (start + size_of::<usize>() - 1) & (!size_of::<usize>() + 1);
        end = end & (!size_of::<usize>() + 1);
        let mut current_start = start;
        while current_start < end {
            let to_alloc = min(lowbit(current_start), prev_power_of_two(end - current_start));
            self.free_list[to_alloc.trailing_zeros() as usize].push(current_start as *mut usize);
            self.size += to_alloc;
        }
    }

    pub unsafe fn init(&mut self, mut start: usize, mut end: usize) {
        self.add_to_head(start, end);
    }

    pub unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, ()> {
        let size = max(
            layout.size().next_power_of_two(),
            max(layout.align(), size_of::usize()));
        let power = size.trailing_zeros() as usize;
        for i in power..self.free_list.len() {
            if !self.free_list[i].is_empty() {
                for j in (power..i).rev() {
                    let new_block = self.free_list[j].pop() as usize;
                    self.free_list[j - 1].push(new_block as *mut usize);
                    self.free_list[j - 1].push((new_block + 1 << (j - 1)) as *mut usize);
                }
            }
        }
        if self.free_list[power].is_empty() {
            Err(())
        }else{

        }
    }
}