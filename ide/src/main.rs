#![no_std]
#![no_main]

extern crate alloc;
use kui::widgets::*;
use kui::*;

use core::panic::PanicInfo;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start(str_ptr: u32, str_len: u32) -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    let new_str = alloc::string::String::from(unsafe {
        core::str::from_utf8_unchecked(core::slice::from_raw_parts(
            str_ptr as *const u8,
            str_len as usize,
        ))
    });

    let mut f = libk::io::File::new(&new_str);

    let b = f.read_bytes();
    let global_str = unsafe { core::str::from_utf8_unchecked(b) };

    let mut rng = libk::rng::LcgRng::new(core::ptr::addr_of!(ALLOC) as u64);
    let r = rng.range(0, 255) as u8;
    let g = rng.range(0, 255) as u8;
    let b = rng.range(0, 255) as u8;

    let mut main = Window::new()
        .name("Ide")
        .width(Size::new("100"))
        .height(Size::new("100"))
        .color(Color::rgb(r, g, b))
        .display(Display::Flex);

    let mut frame = Widget::Frame(
        Frame::new()
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .color(Color::rgb(255, 255, 255)),
    );

    let tf = Widget::InputLabel(
        Label::new()
            .text(global_str)
            .color(Color::rgb(255, 255, 255))
            .text_color(Color::rgb(0, 0, 0))
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .min(0),
    );

    frame.add(tf);
    main.add(frame);

    kui::draw::init(main);

    loop {}
}

/*pub fn list_entries(dir: &str) {
    let mut count = 0;

    loop {
        let e = libk::io::get_entry(dir, count);
        if e.is_none() { return; }

        libk::println!("{}{}", dir, unsafe { core::str::from_utf8_unchecked(&e.unwrap().name) });

        count += 1;
    }
}*/

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
