use crate::println;
use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicPtr, Ordering};

#[repr(C, packed(8))]
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
        self.size = unsafe {
            end.offset_from(self.get_start())
                .try_into()
                .expect("Expected end > start")
        };
    }
}

#[repr(C, packed(8))]
#[derive(Debug)]
struct UsedSegment {
    size: usize,
}

impl UsedSegment {
    fn get_start(&self) -> *mut u8 {
        unsafe { (self as *const UsedSegment).add(1) as *mut u8 }
    }

    fn get_end(&self) -> *mut u8 {
        unsafe { self.get_start().add(self.size) }
    }

    fn set_end(&mut self, end: *mut u8) {
        unsafe {
            self.size = end
                .offset_from(self.get_start())
                .try_into()
                .expect("Expected end > start");
        }
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

    pub fn init(&self, size: usize) {
        let addr = crate::syscall::malloc(size as u32);

        let segment_size: usize = size - core::mem::size_of::<FreeSegment>();

        let segment = addr as *mut FreeSegment;

        unsafe {
            *segment = FreeSegment {
                size: segment_size,
                next_segment: core::ptr::null_mut(),
            };
        }

        self.first_free.store(segment, Ordering::SeqCst);
    }
}

fn find_header_for_allocation(segment: &FreeSegment, layout: &Layout) -> Option<*mut u8> {
    let segment_start: *mut u8 = segment.get_start();
    let segment_end: *mut u8 = segment.get_end();

    let header_size = core::mem::size_of::<UsedSegment>();

    let total_size = header_size + layout.size();

    if segment.size < total_size {
        return None;
    }

    let mut data_end = segment_end;

    let unaligned_data_start = unsafe { data_end.sub(layout.size()) };

    let align_correction = (unaligned_data_start as usize) % layout.align();
    let aligned_data_start = if align_correction > 0 {
        unsafe { unaligned_data_start.sub(align_correction) }
    } else {
        unaligned_data_start
    };

    let header_ptr = unsafe { aligned_data_start.sub(header_size) };

    if header_ptr < segment_start {
        println!(
            "Segment size too small, segment: {:?}, layout: {:?}",
            segment, layout
        );
        return None;
    }

    Some(header_ptr)
}

fn get_header_ptr_from_allocated(ptr: *mut u8) -> *mut UsedSegment {
    unsafe { ptr.sub(core::mem::size_of::<UsedSegment>()) as *mut UsedSegment }
}

fn merge_if_adjacent(a: *mut FreeSegment, b: *mut FreeSegment) {
    if b.is_null() {
        return;
    }

    unsafe {
        if (*a).get_end() == b as *mut u8 {
            (*a).set_end((*b).get_end());
            (*a).next_segment = (*b).next_segment;
        }
    }
}

fn insert_segment_after(item: *mut FreeSegment, new_segment: *mut FreeSegment) {
    unsafe {
        let next = (*item).next_segment;
        (*item).next_segment = new_segment;
        (*new_segment).next_segment = next;
        merge_if_adjacent(new_segment, (*new_segment).next_segment);
        merge_if_adjacent(item, new_segment);
    }
}

fn insert_segment_into_list(first_free: &AtomicPtr<FreeSegment>, new_segment: *mut FreeSegment) {
    let mut current_head = first_free.load(Ordering::SeqCst);
    if current_head.is_null() || new_segment < current_head {
        loop {
            unsafe { (*new_segment).next_segment = current_head };
            match first_free.compare_exchange_weak(
                current_head,
                new_segment,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => {
                    if !current_head.is_null() {
                        merge_if_adjacent(new_segment, current_head);
                    }
                    return;
                }
                Err(new_head) => {
                    current_head = new_head;
                    if current_head.is_null() || new_segment < current_head {
                        continue;
                    }
                    break;
                }
            }
        }
    }
    let mut it = current_head;
    let mut prev = core::ptr::null_mut();
    while !it.is_null() && it < new_segment {
        prev = it;
        it = unsafe { (*it).next_segment };
    }

    if prev.is_null() {
        panic!("Logic error in insert_segment_into_list");
    }

    insert_segment_after(prev, new_segment);
}

fn convert_used_to_free_segment(first_free: &AtomicPtr<FreeSegment>, header_ptr: *mut UsedSegment) {
    unsafe {
        let size = (*header_ptr).size;
        let free_segment_ptr = header_ptr as *mut FreeSegment;
        (*free_segment_ptr).size = size;
        (*free_segment_ptr).next_segment = core::ptr::null_mut();
        insert_segment_into_list(first_free, free_segment_ptr);
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let layout = if layout.align() <= 1 {
            Layout::from_size_align(layout.size(), core::mem::align_of::<usize>())
                .expect("Failed to create aligned layout")
        } else {
            layout
        };
        loop {
            let current_head = self.first_free.load(Ordering::SeqCst);
            if current_head.is_null() {
                panic!("Out of memory: Failed to allocate");
            }

            let mut prev_ptr: *mut FreeSegment = core::ptr::null_mut();
            let mut free_block_it = current_head;
            while !free_block_it.is_null() {
                let header_ptr = find_header_for_allocation(&*free_block_it, &layout);

                match header_ptr {
                    Some(header_ptr) => {
                        let segment = &mut *free_block_it;
                        let used_end = segment.get_end();
                        let header = header_ptr as *mut UsedSegment;
                        (*header).set_end(used_end);
                        segment.set_end(header_ptr);
                        if segment.size < core::mem::size_of::<FreeSegment>() {
                            if prev_ptr.is_null() {
                                if !self
                                    .first_free
                                    .compare_exchange(
                                        current_head,
                                        segment.next_segment,
                                        Ordering::SeqCst,
                                        Ordering::SeqCst,
                                    )
                                    .is_ok()
                                {
                                    break;
                                }
                            } else {
                                (*prev_ptr).next_segment = segment.next_segment;
                            }
                        }

                        return (*header).get_start();
                    }
                    None => {
                        prev_ptr = free_block_it;
                        free_block_it = (*free_block_it).next_segment;
                    }
                }
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let header_ptr = get_header_ptr_from_allocated(ptr);
        convert_used_to_free_segment(&self.first_free, header_ptr);
    }
}
