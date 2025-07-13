#![no_std]
#![no_main]

extern crate alloc;

use alloc::vec::Vec;
use bafioDb;
use core::panic::PanicInfo;
use kui::draw::*;
use kui::widgets::*;
use libk::mutex::Mutex;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    libk::println!("OS started !");

    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    let mut wallpaper = Window::new()
        .name("S")
        .width(Size::new("100%"))
        .height(Size::new("100%"))
        .color(Color::rgb(124, 147, 195))
        .action_bar(false)
        .w_type(libk::syscall::Items::Wallpaper);

    let wp_i = Widget::Image(
        Image::new("SYS/BG.TGA")
            .height(Size::new("100%"))
            .width(Size::new("100%")),
    );

    let sz = unsafe { (*(&raw mut SCREEN)).height.absolute.unwrap() };

    let y = kui::kui_ceil(sz as f32 - (sz as f32 * 8.0 / 100.0)) as u32;
    let h = kui::kui_ceil(sz as f32 * 8.0 / 100.0) as u32;

    let mut action_bar = Window::new()
        .y(Size::from_u32(y))
        .width(Size::new("100%"))
        .height(Size::from_u32(h))
        .color(Color::rgb(56, 75, 112))
        .action_bar(false)
        .display(Display::Flex)
        .w_type(libk::syscall::Items::Bar);

    wallpaper.add(wp_i);

    let f = bafioDb::load("/SYS/ICONS.DB");

    for x in 0..list_entries("/USER/DESKTOP") {
        let e = &libk::io::get_entry("/USER/DESKTOP", x).unwrap().name;
        let fname = alloc::string::String::from(unsafe { core::str::from_utf8_unchecked(e) });
        let func = alloc::string::String::from("/USER/DESKTOP/") + &fname;

        unsafe {
            (*(&raw mut PROGRAMS)).lock().push(func.clone());

            let mut file_icon = "ICONS/FILE.TGA";
            let f3 = libk::io::File::new(&func);
            if f3.is_dir() {
                file_icon = "ICONS/FOLDER2.TGA";
            } else {
                let icon_path = f.get(f3.get_file_extention());

                if icon_path.is_some() {
                    match icon_path.unwrap() {
                        crate::bafioDb::Value::String(s) => {
                            let owned_file_icon = s.clone();
                            file_icon = alloc::boxed::Box::leak(owned_file_icon.into_boxed_str());
                        }
                        _ => {}
                    }
                }
            }

            let img_h = kui::kui_ceil(h as f32 * 65.0 / 100.0) as u32;

            let i = Widget::Image(
                Image::new(&file_icon)
                    .height(Size::from_u32(img_h))
                    .width(Size::from_u32(img_h))
                    .event(start_file)
                    .set_args([((*(&raw mut PROGRAMS)).lock().len() - 1) as u32, 0, 0]),
            );

            action_bar.add(i);
        }
    }

    init(wallpaper);
    init(action_bar);

    /*let sock = libk::net::Socket::new(68);
    sock.send_dhcp_discover();
    let n = sock.recv(1024);
    sock.handle_dhcp(unsafe { &*(n.as_ptr() as *const libk::packets::DhcpPacket) });
    let n = sock.recv(1024);
    sock.handle_dhcp(unsafe { &*(n.as_ptr() as *const libk::packets::DhcpPacket) });
    sock.close();*/

    libk::println!("[-]");

    loop {}
}

pub static mut PROGRAMS: Mutex<Vec<alloc::string::String>> = Mutex::new(Vec::new());

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

pub fn start_file(_w: &mut Widget, a1: u32, _a2: u32, _a3: u32) {
    unsafe {
        let p = (*(&raw mut PROGRAMS)).lock();
        /*let t = "/USER/DESKTOP/PROC1   ELF";
        let str_addr = t.as_ptr() as u32;
        let str_len = t.len() as u32;*/

        let _ = libk::elf::load_elf(&p[a1 as usize], Some(&[0, 0, 0, 0]));
    }
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
