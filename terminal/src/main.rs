#![no_std]
#![no_main]

extern crate alloc;
use kui::widgets::*;
use kui::*;

use core::panic::PanicInfo;
use kui::draw::draw_label;
use libk::println;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    libk::println!("Reading stuff");

    let mut rng = libk::rng::LcgRng::new(core::ptr::addr_of!(ALLOC) as u64);
    let r = rng.range(0, 255) as u8;
    let g = rng.range(0, 255) as u8;
    let b = rng.range(0, 255) as u8;

    let mut main = Window::new()
        .name("Terminal")
        .width(Size::new("100"))
        .height(Size::new("100"))
        .color(Color::rgb(r, g, b))
        .display(Display::Flex);

    let mut frame = Widget::Frame(
        Frame::new()
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .color(Color::rgb(0, 0, 0)),
    );

    let tf = Widget::InputLabel(
        Label::new()
            .text("\n bafiOS@guest> ")
            .color(Color::rgb(0, 0, 0))
            .text_color(Color::rgb(255, 255, 255))
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .min(16),
    );

    unsafe {
        (*(&raw mut draw::KEY_MAP)).add('\n', test);
    }

    frame.add(tf);
    main.add(frame);

    draw::init(main);

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
        println!("{}{}", dir, unsafe {
            core::str::from_utf8_unchecked(e_var)
        });

        count += 1;
    }
}

pub fn test(w: &mut Widget) {
    if let Widget::InputLabel(l) = w {
        let base_str = "\n bafiOS@guest> ";
        let cmd = alloc::string::String::from(&l.label[l.ch_min as usize..]);

        drop(l);

        parse(w, &cmd);
        if let Widget::InputLabel(l) = w {
            if l.label.len() < (l.ch_max as usize - base_str.len()) {
                l.label.push_str(base_str);
                l.ch_min = l.label.len() as u32;
                draw_label(l);
            }
        }
    }
}

pub fn parse(w: &mut Widget, command: &str) {
    use alloc::vec::Vec;

    let commands: Vec<&str> = command.split_whitespace().collect();

    if let Some(first_command) = commands.first() {
        match w {
            Widget::InputLabel(l) => {
                match *first_command {
                    "echo" => {
                        if commands.len() > 1 {
                            let output = commands[1..].join(" ");
                            l.label.push('\n');
                            l.label.push('\n');
                            l.label.push(' ');
                            l.label.push_str(&output);
                            l.label.push('\n');
                        }
                    }

                    "pwd" => {}

                    "ls" => {}

                    "cd" => {}

                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
