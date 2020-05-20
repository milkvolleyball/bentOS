use alloc::alloc::{GlobalAlloc,Layout};
use super::{Locked, align_up};
use core::ptr;

pub struct BumpAllocator {
    heap_start:usize,
    heap_end:usize,
    next:usize, //always point to the first unused byte of heap. the start address of next allocation.
    allocations:usize,
}impl BumpAllocator {
    ///Creates a new empty bump allocator.
    pub const fn new()->Self {
        BumpAllocator {
            heap_start: 0,
            heap_end:0,
            next:0,
            allocations:0,
        }
    }
    /// Initializes the bump allocator with the given heap bounds.
    /// This method is unsafe because the caller must ensure that the given
    /// memory range is unused. Also, this method must be called only once.

    pub unsafe fn init(&mut self, heap_start:usize, heap_size:usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

//alloc and dealloc need a &self. but we have to modify our allocator.
//so we put our allocator into spin::Mutex to get interior mutability.
//but we cant use spin::Mutex directly because spin::mutex::Mutex is not defined in the current crate
//so we wrap it into a new structure"Locked" which defined by ourself.
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    unsafe fn alloc(&self,layout:Layout) -> *mut u8 {
        let mut bump = self.lock();//get a mutable reference.
        //alloc_start stand for the start address of next allocation
        let alloc_start = align_up(bump.next, layout.align());
        //layout.size() gives the minimum size in bytes for a memory block of current layout.
        //checked_add() checks is it possible to allocate a minimum size of a block to next allocation's start address.
        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end{
            ptr::null_mut() //out of mem. no allocate.
        } else {
            bump.next = alloc_end;//update next to point at the end address of the allocation
            bump.allocations += 1;
            
            alloc_start as *mut u8 //return start address of the allocation as *mut u8 pointer
        }

    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let mut bump = self.lock();//get a mutable ref
        bump.allocations -= 1;
        if bump.allocations == 0{
            bump.next = bump.heap_start;
        }
    }
}