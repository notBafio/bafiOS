#![no_std]
#![no_main]

extern crate alloc;

use core::panic::PanicInfo;
use kui::widgets::*;

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

    let mut main = Window::new()
        .name("Img")
        .width(Size::new("500"))
        .height(Size::new("500"))
        .color(Color::rgb(255, 255, 255))
        .display(Display::None);

    let mut f = Widget::Frame(
        Frame::new()
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .color(Color::rgb(255, 255, 255))
            .display(Display::Flex),
    );

    let tf = Widget::Image(
        Image::new(&new_str)
            .width(Size::new("80%"))
            .height(Size::new("80%")),
    );

    f.add(tf);
    main.add(f);

    kui::draw::init(main);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
