#![no_std]
#![no_main]

extern crate alloc;
use kui::widgets::*;
use kui::*;
use libk::packets::DhcpPacket;

use core::panic::PanicInfo;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    let mut rng = libk::rng::LcgRng::new(core::ptr::addr_of!(ALLOC) as u64);
    let r = rng.range(0, 255) as u8;
    let g = rng.range(0, 255) as u8;
    let b = rng.range(0, 255) as u8;

    let main = Window::new()
        .name("Hell")
        .width(Size::new("100"))
        .height(Size::new("100"))
        .color(Color::rgb(r, g, b))
        .display(Display::Flex);

    list_entries("/");

    kui::draw::init(main);

    loop {}
}

pub fn list_entries(dir: &str) {
    let mut count = 0;

    loop {
        let e = libk::io::get_entry(dir, count);
        if e.is_none() {
            return;
        }

        let e_var = &e.unwrap().name;
        libk::println!("{}{}", dir, unsafe {
            core::str::from_utf8_unchecked(e_var)
        });

        count += 1;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
