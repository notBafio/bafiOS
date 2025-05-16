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

use alloc::string::String;
struct Terminal {
    path: String,
    user: String,
}

pub static mut TERMINAL: Terminal = Terminal {
    path : String::new(),
    user: String::new(),
};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    libk::println!("Reading stuff");

    unsafe {
        TERMINAL.path = String::from("/");
        TERMINAL.user = String::from("guest");
    }

    let mut main = Window::new()
        .name("Terminal")
        .width(Size::new("200"))
        .height(Size::new("150"))
        .color(Color::rgb(0, 0, 0))
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
        'exit_match: {
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

                        "pwd" => {
                            l.label.push('\n');
                            l.label.push('\n');
                            l.label.push(' ');
                            unsafe { l.label.push_str(&(*(&raw mut TERMINAL)).path); }
                            l.label.push('\n');
                        }

                        "ls" => {
                            unsafe {
                                l.label.push('\n');

                                for x in 0..list_entries(&(*(&raw mut TERMINAL)).path) {
                                    let e = &libk::io::get_entry(&(*(&raw mut TERMINAL)).path, x).unwrap().name;
                                    let fname = alloc::string::String::from(core::str::from_utf8_unchecked(e));

                                    l.label.push('\n');
                                    l.label.push(' ');
                                    l.label.push_str(&fname);
                                }

                                l.label.push('\n');
                            }
                        }

                        "cd" => {
                            let terminal = unsafe { &mut *(&raw mut TERMINAL) };

                            if commands.len() <= 1 { break 'exit_match; }

                            let target_dir = commands[1];

                            if target_dir == ".." {
                                let path = &terminal.path;

                                let search_end = if path.ends_with('/') { path.len() - 1 } else { path.len() };

                                if let Some(last_slash) = path[..search_end].rfind('/') {
                                    terminal.path.truncate(last_slash + 1);
                                    break 'exit_match;
                                } else {
                                    terminal.path = alloc::string::String::from("/");
                                    break 'exit_match;
                                }
                            }

                            let mut new_path = terminal.path.clone();

                            if !new_path.ends_with('/') {
                                new_path.push('/');
                            }

                            new_path.push_str(target_dir);

                            if !new_path.ends_with('/') {
                                new_path.push('/');
                            }

                            let entries_count = list_entries(&terminal.path);
                            let mut dir_exists = false;

                            for x in 0..entries_count {
                                let entry = match libk::io::get_entry(&terminal.path, x) {
                                    Some(entry) => entry,
                                    None => continue,
                                };

                                let entry_name = unsafe {
                                    alloc::string::String::from(core::str::from_utf8_unchecked(&entry.name))
                                };

                                if entry_name.trim_end() == target_dir && entry.is_dir() {
                                    dir_exists = true;
                                    break;
                                }
                            }

                            if dir_exists {
                                terminal.path = new_path;
                            }
                        }

                        "exec" => {}

                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn list_entries(dir: &str) -> u8 {
    let mut count = 0;

    loop {
        if count == 255 {
            return count;
        }
        let e = libk::io::get_entry(dir, count);
        if e.is_none() {
            return count;
        }

        let entry = e.unwrap();
        let s = unsafe { core::str::from_utf8_unchecked(&entry.name) };
        let new_name = expand_path_8_3(s);

        count += 1;
    }
}
#[inline]
fn is_whitespace(b: u8) -> bool {
    b == b' ' || b == b'\t' || b == b'\n' || b == b'\r'
}


pub fn expand_path_8_3(path: &str) -> &[u8] {
    assert!(path.len() == 11, "Path must be exactly 11 characters long");

    if !path.contains(' ') {
        return path.as_bytes();
    }

    static mut BUFFER: [u8; 12] = [0; 12];

    let name_part = &path[..8];
    let ext_part = &path[8..];

    let trimmed_name = name_part.trim_end();
    let trimmed_ext = ext_part.trim_end();

    unsafe {
        let mut idx = 0;

        for byte in trimmed_name.bytes() {
            BUFFER[idx] = byte;
            idx += 1;
        }

        if !trimmed_ext.is_empty() {
            BUFFER[idx] = b'.';
            idx += 1;

            for byte in trimmed_ext.bytes() {
                BUFFER[idx] = byte;
                idx += 1;
            }
        }

        &BUFFER[..idx]
    }
}