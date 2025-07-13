#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start(str_ptr: u32, str_len: u32) {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    let new_str = unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            str_ptr as *const u8,
            str_len as usize,
        ))
    };

    libk::println!("Running... {}", new_str);

    let _ = libk::elf::load_elf(new_str, None);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
