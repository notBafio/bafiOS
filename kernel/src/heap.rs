use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicPtr, Ordering};
use libk::println;

const MIN_FREE_SEGMENT_SIZE: usize = 64;
const USED_SEGMENT_MAGIC: u32 = 0xBAF10500;

#[repr(C, packed)]
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct FreeSegment {
    size: usize,
    next_segment: *mut FreeSegment,
}

impl FreeSegment {
    fn get_start(&self) -> *mut u8 {
        unsafe { (self as *const FreeSegment).add(1) as *mut u8 }
    }

    fn get_end(&self) -> *mut u8 {
        unsafe { self.get_start().add(self.size) }
    }

    fn set_end(&mut self, end: *mut u8) {
        let diff = unsafe { end.offset_from(self.get_start()) };
        if diff <= 0 {
            panic!("end < start");
        }
        self.size = diff as usize;
    }

    fn can_fit(&self, layout: &Layout) -> bool {
        let header_size = core::mem::size_of::<UsedSegment>();
        let total_needed = header_size + layout.size() + layout.align() - 1;
        self.size >= total_needed
    }
}

#[repr(C, packed)]
#[derive(Debug)]
struct UsedSegment {
    size: usize,
    magic: u32,
}

impl UsedSegment {
    fn get_start(&self) -> *mut u8 {
        unsafe { (self as *const UsedSegment).add(1) as *mut u8 }
    }

    fn set_end(&mut self, end: *mut u8) {
        let diff = unsafe { end.offset_from(self.get_start()) };
        if diff <= 0 {
            panic!("end < start");
        }
        self.size = diff as usize;
    }

    fn init(&mut self, size: usize) {
        self.size = size;
        self.magic = USED_SEGMENT_MAGIC;
    }

    fn is_valid(&self) -> bool {
        self.magic == USED_SEGMENT_MAGIC
    }
}

pub struct Allocator {
    pub first_free: AtomicPtr<FreeSegment>,
}

impl Allocator {
    pub const fn new() -> Allocator {
        Allocator {
            first_free: AtomicPtr::new(core::ptr::null_mut()),
        }
    }

    pub fn init(&self) {
        unsafe {
            let segment_size: usize = 0x10_0000 - core::mem::size_of::<FreeSegment>();
            let segment = 0x30_0000 as *mut FreeSegment;
            *segment = FreeSegment {
                size: segment_size,
                next_segment: core::ptr::null_mut(),
            };
            self.first_free.store(segment, Ordering::Release);
        }
    }
}

fn find_header_for_allocation(segment: &FreeSegment, layout: &Layout) -> Option<*mut u8> {
    let segment_start = segment.get_start() as usize;
    let segment_end = segment.get_end() as usize;

    let header_size = core::mem::size_of::<UsedSegment>();
    let payload_size = layout.size();
    if segment_end - segment_start < header_size + payload_size {
        return None;
    }
    let max_payload_start = segment_end - payload_size;
    let aligned_payload_start = max_payload_start & !(layout.align() - 1);
    let header_ptr = aligned_payload_start - header_size;

    if header_ptr < segment_start {
        println!(
            "After alignment, segment too small. segment_start: {:#x}, header_ptr: {:#x}, segment_end: {:#x}",
            segment_start, header_ptr, segment_end
        );
        return None;
    }
    let free_space = header_ptr - segment_start;
    if free_space > 0 && free_space < MIN_FREE_SEGMENT_SIZE {
        if header_ptr < segment_start + MIN_FREE_SEGMENT_SIZE {
            return None;
        }
        return Some((header_ptr - MIN_FREE_SEGMENT_SIZE) as *mut u8);
    }

    Some(header_ptr as *mut u8)
}
fn get_header_ptr_from_allocated(ptr: *mut u8) -> *mut UsedSegment {
    unsafe { ptr.sub(core::mem::size_of::<UsedSegment>()) as *mut UsedSegment }
}
fn merge_if_adjacent(a: *mut FreeSegment, b: *mut FreeSegment) -> bool {
    if a.is_null() || b.is_null() {
        return false;
    }
    unsafe {
        if (*a).get_end() == b as *mut u8 {
            (*a).set_end((*b).get_end());
            (*a).next_segment = (*b).next_segment;
            return true;
        }
    }
    false
}
fn insert_segment_after(item: *mut FreeSegment, new_segment: *mut FreeSegment) {
    unsafe {
        let next = (*item).next_segment;
        (*item).next_segment = new_segment;
        (*new_segment).next_segment = next;

        if !next.is_null() {
            merge_if_adjacent(new_segment, next);
        }
        merge_if_adjacent(item, new_segment);
    }
}
fn insert_segment_into_list(list_head: *mut FreeSegment, new_segment: *mut FreeSegment) {
    if list_head.is_null() {
        panic!("Cannot insert into an empty free list");
    }
    unsafe {
        let mut it = list_head;
        while !it.is_null() {
            let next = (*it).next_segment;
            if next.is_null() || (next as usize) > (new_segment as usize) {
                insert_segment_after(it, new_segment);
                return;
            }
            it = next;
        }
        panic!("Failed to insert segment into free list");
    }
}
fn convert_used_to_free_segment(list_head: *mut FreeSegment, header_ptr: *mut UsedSegment) {
    unsafe {
        if !(*header_ptr).is_valid() {
            println!("WARNING: Detected invalid memory segment during deallocation!");
            return;
        }
        let size = (*header_ptr).size;
        let free_segment_ptr = header_ptr as *mut FreeSegment;
        (*free_segment_ptr).size = size;
        (*free_segment_ptr).next_segment = core::ptr::null_mut();
        insert_segment_into_list(list_head, free_segment_ptr);
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == 0 {
            return NonNull::dangling().as_ptr();
        }
        let mut free_block_it = self.first_free.load(Ordering::Acquire);
        let mut prev: *mut FreeSegment = core::ptr::null_mut();

        while !free_block_it.is_null() {
            if let Some(header_ptr) = find_header_for_allocation(&*free_block_it, &layout) {
                let used_end = (*free_block_it).get_end();
                (*free_block_it).set_end(header_ptr);
                if (*free_block_it).size < MIN_FREE_SEGMENT_SIZE {
                    let next = (*free_block_it).next_segment;
                    if prev.is_null() {
                        self.first_free.store(next, Ordering::Release);
                    } else {
                        (*prev).next_segment = next;
                    }
                }
                let used_header_ptr = header_ptr as *mut UsedSegment;
                let payload_size = (used_end as usize)
                    - (used_header_ptr as usize + core::mem::size_of::<UsedSegment>());
                (*used_header_ptr).init(payload_size);
                return (*used_header_ptr).get_start();
            } else {
                prev = free_block_it;
                free_block_it = (*free_block_it).next_segment;
            }
        }
        println!("ALLOCATION FAILED: Out of memory, layout: {:?}", layout);
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() || ptr == NonNull::<u8>::dangling().as_ptr() {
            return;
        }
        let header_ptr = get_header_ptr_from_allocated(ptr);
        let list_head = self.first_free.load(Ordering::Acquire);
        if list_head.is_null() {
            (*header_ptr).magic = 0;
            let free_ptr = header_ptr as *mut FreeSegment;
            (*free_ptr).next_segment = core::ptr::null_mut();
            self.first_free.store(free_ptr, Ordering::Release);
        } else {
            convert_used_to_free_segment(list_head, header_ptr);
        }
    }
}
