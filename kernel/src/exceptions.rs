use crate::keyboard;
use crate::pic::PICS;
use libk::port::{inb, outb};
use libk::println;

use crate::composer::{COMPOSER, Window};
use core::arch::{asm, naked_asm};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    err_code: u32,
    eip: u32,
    cs: u32,
    flags: u32,
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct StackFrame {
    eip: u32,
    cs: u32,
    flags: u32,
}

pub extern "x86-interrupt" fn div_error(info: StackFrame) {
    println!("DIVISION ERROR!");

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn bounds(info: StackFrame) {
    println!("OUT OF BOUNDS! -> {:?}", info);

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn invalid_opcode(info: StackFrame) {
    println!("IO -> {:?}", info);

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn double_fault(info: StackFrame) {
    println!("DOUBLE FAULT!");

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn general_protection_fault(info: InterruptStackFrame) {
    println!("GPF -> {:?}", info);

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn page_fault(info: InterruptStackFrame) {
    println!("PAGE FAULT!");

    if info.cs == 0x1B {
        crate::task::exit();
    }
}

pub extern "x86-interrupt" fn generic_handler() {
    println!("EXCEPTION!");
}

/* SPECIFIC STUFF */

pub const NET_INT: u8 = 43;

pub const TIMER_INT: u8 = 32;

pub const KEYBOARD_INT: u8 = 33;

pub extern "x86-interrupt" fn keyboard_handler() {
    libk::disable_interrupts();

    let data: u8 = inb(0x60);

    keyboard::keyboard_italian(data);
    libk::print!("<{}>", data);

    unsafe {
        (*(&raw mut PICS)).end_interrupt(KEYBOARD_INT);
    }

    libk::enable_interrupts();
}

pub const MOUSE_INT: u8 = 44;
pub static mut MOUSE_PACKET: [u8; 3] = [0; 3];
pub static mut MOUSE_IDX: usize = 0;

pub extern "x86-interrupt" fn mouse_handler() {
    unsafe {
        libk::disable_interrupts();

        let data = inb(0x60);

        if MOUSE_IDX == 0 && (data & 0b00001000) == 0 {
            (*(&raw mut PICS)).end_interrupt(MOUSE_INT);
            libk::enable_interrupts();
            return;
        }

        MOUSE_PACKET[MOUSE_IDX] = data;
        MOUSE_IDX += 1;

        if MOUSE_IDX == 3 {
            (*(&raw mut crate::composer::MOUSE)).cursor(MOUSE_PACKET);
            MOUSE_IDX = 0;
        }

        (*(&raw mut PICS)).end_interrupt(MOUSE_INT);
        libk::enable_interrupts();
    }
}

pub const RTC_INT: u8 = 40;

pub extern "x86-interrupt" fn rtc_handler() {
    outb(0x70, 0x0C);
    let _ = inb(0x71);

    unsafe {
        (*(&raw mut PICS)).end_interrupt(RTC_INT);
    }
}

#[naked]
pub extern "C" fn syscall() {
    unsafe {
        naked_asm!(
            "cli",
            "push edx",
            "push ecx",
            "push ebx",
            "push eax",
            "call syscall_handler",
            "add esp, 16",
            "push eax",
            "pop eax",
            "sti",
            "iretd",
        );
    }
}

pub static mut W_DATA: (u32, u32) = (0, 0);
pub static mut TEMP_FILE: crate::fs::fat16::structs::Entry = crate::fs::fat16::structs::NULL_ENTRY;

#[inline(never)]
#[unsafe(no_mangle)]
pub extern "C" fn syscall_handler(eax: u32, ebx: u32, ecx: u32, edx: u32) -> u32 {
    unsafe {
        let mut return_val = 0;

        match eax {
            0 => {
                let base = core::ptr::addr_of!(crate::BOOTINFO) as u32;
                return_val = base;
            }

            1 => {
                println!("TEST");
            }

            2 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                if let Some(entry) = (*(&raw mut crate::fs::fat16::structs::FAT16))
                    .find_entry(&format_path_8_3(filename))
                {
                    (*(&raw mut crate::fs::fat16::structs::FAT16)).read(&entry, ecx as *mut u8);
                    return_val = 1;
                }
            }

            3 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                if let Some(entry) = (*(&raw mut crate::fs::fat16::structs::FAT16))
                    .find_entry(&format_path_8_3(filename))
                {
                    TEMP_FILE = entry;
                    return_val = core::ptr::addr_of!(TEMP_FILE) as u32;
                }
            }

            4 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                if let Some(entry) = (*(&raw mut crate::fs::fat16::structs::FAT16))
                    .find_entry(&format_path_8_3(&format_path_8_3(filename)))
                {
                    return_val = entry.size;
                } else {
                    libk::println!("[!] {} 404 not found", filename);
                    return_val = 699669;
                }
            }

            5 => {
                let base = (*(&raw mut crate::pmm::PADDR)).malloc(ebx);

                return_val = base.unwrap();
            }

            6 => {
                (*(&raw mut crate::pmm::PADDR)).dealloc(ebx);
            }

            7 => {
                return_val = (*(&raw mut crate::composer::DISPLAY_SERVER)).double_buffer as u32;
            }

            8 => {
                let c = *(ecx as *const Coordiates);
                (*(&raw mut crate::composer::DISPLAY_SERVER))
                    .copy_to_db(c.w as u32, c.h as u32, ebx as u32, c.x as u32, c.y as u32);
                (*(&raw mut crate::composer::DISPLAY_SERVER)).copy();
                return_val = 1;
            }

            9 => {
                for j in (0..(*(&raw mut crate::composer::COMPOSER)).windows.len()).rev() {
                    match (*(&raw mut crate::composer::COMPOSER)).windows[j].wtype {
                        crate::composer::Items::Null => {}
                        _ => {
                            (*(&raw mut crate::composer::DISPLAY_SERVER)).copy_to_db(
                                (*(&raw mut crate::composer::COMPOSER)).windows[j].width as u32,
                                (*(&raw mut crate::composer::COMPOSER)).windows[j].height as u32,
                                (*(&raw mut crate::composer::COMPOSER)).windows[j].buffer as u32,
                                (*(&raw mut crate::composer::COMPOSER)).windows[j].x as u32,
                                (*(&raw mut crate::composer::COMPOSER)).windows[j].y as u32,
                            );
                        }
                    }
                }

                (*(&raw mut crate::composer::DISPLAY_SERVER)).copy();
                return_val = 1;
            }

            10 => {}

            22 => {
                let w = *(ebx as *const Window);
                W_DATA = (*(&raw mut crate::composer::COMPOSER)).add_window(w);

                return_val = core::ptr::addr_of!(W_DATA) as u32;
            }

            23 => {
                (*(&raw mut crate::composer::COMPOSER)).remove_window(ebx as u16);
                return_val = 1;
            }

            25 => {
                let mut args_ptr: Option<&[u32]> = None;

                if edx != 0 {
                    args_ptr = Some(core::slice::from_raw_parts(edx as *const u32, 4));
                }

                (*(&raw mut crate::task::TASK_MANAGER))
                    .lock()
                    .add_user_task(ebx, args_ptr);
                return_val = 1;
            }

            26 => {
                crate::task::exit();
            }

            27 => {
                (*(&raw mut crate::net::rtl8139::RTL8139)).send_tcp_syn(
                    6969,
                    [0xff; 6],
                    [142, 250, 180, 174],
                    80,
                    0xfe55a,
                );
            }

            28 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let n_entries = (*(&raw mut crate::fs::fat16::structs::FAT16)).count_entries_in_dir(
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]),
                );

                return_val = n_entries;
            }

            29 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let mut n_entries = (*(&raw mut crate::fs::fat16::structs::FAT16)).get_entries_by_id(
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]),
                    ecx as u8,
                );

                if n_entries.is_some() {
                    let e_size = core::mem::size_of::<crate::fs::fat16::structs::Entry>() as u32;

                    let addr = (*(&raw mut crate::pmm::PADDR)).malloc(e_size).unwrap();

                    *(addr as *mut crate::fs::fat16::structs::Entry) = n_entries.unwrap();

                    return_val = addr;
                } else {
                    return_val = 0;
                }
            }

            30 => {
                let packet_addr = ebx as *const u8;
                let packet_size = ecx as usize;

                (*(&raw mut crate::net::rtl8139::RTL8139))
                    .send_clean_packet(core::slice::from_raw_parts(packet_addr, packet_size));
            }

            31 => {
                (*(&raw mut crate::net::socket::SOCKETS)).new(ebx as u16, ecx);
            }

            32 => {
                (*(&raw mut crate::net::socket::SOCKETS)).close(ebx as u16);
            }

            33 => {
                return_val = core::ptr::addr_of!(crate::net::rtl8139::RTL8139) as u32;
            }

            34 => {
                (*(&raw mut crate::net::rtl8139::RTL8139)).ip =
                    core::ptr::read(ebx as *const [u8; 4]);
            }

            35 => {
                (*(&raw mut crate::net::rtl8139::RTL8139)).dns =
                    core::ptr::read(ebx as *const [u8; 4]);
            }

            36 => {
                (*(&raw mut crate::net::rtl8139::RTL8139)).gateway =
                    core::ptr::read(ebx as *const [u8; 4]);
            }

            37 => {
                (*(&raw mut crate::net::rtl8139::RTL8139)).subnet =
                    core::ptr::read(ebx as *const [u8; 4]);
            }

            38 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                if let Some(_entry) = (*(&raw mut crate::fs::fat16::structs::FAT16))
                    .find_entry(&format_path_8_3(filename))
                {
                    let data_ptr = *(ecx as *const (u32, u32));

                    let data =
                        core::slice::from_raw_parts(data_ptr.0 as *const u8, data_ptr.1 as usize);
                    let _ = (*(&raw mut crate::fs::fat16::structs::FAT16))
                        .overwrite_file(&format_path_8_3(filename), data);
                }
            }

            39 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                if let Some(_entry) = (*(&raw mut crate::fs::fat16::structs::FAT16))
                    .find_entry(&format_path_8_3(filename))
                {
                    let data_ptr = *(ecx as *const (u32, u32));
                    let data =
                        core::slice::from_raw_parts(data_ptr.0 as *const u8, data_ptr.1 as usize);
                    let _ = (*(&raw mut crate::fs::fat16::structs::FAT16))
                        .append_to_file(&format_path_8_3(filename), data);
                }
            }

            40 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename = core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                (*(&raw mut crate::fs::fat16::structs::FAT16)).create_file(&format_path_8_3(filename));
            }

            41 => {
                let id = ebx as u16;

                (*(&raw mut COMPOSER)).copy_window(id);
                (*(&raw mut COMPOSER)).copy_window_fb(id);
            }

            42 => {
                (*(&raw mut crate::fs::fat16::structs::FAT16)).reload();

                let mut fname = [0u8; 256];
                for i in 0..(core::cmp::min(edx as usize, 256)) {
                    fname[i] = *((ebx + i as u32) as *mut u8);
                }

                let filename =
                    core::str::from_utf8_unchecked(&fname[..core::cmp::min(edx as usize, 256)]);

                (*(&raw mut crate::fs::fat16::structs::FAT16)).create_dir(&format_path_8_3(filename));
            }

            100 => loop {},

            _ => {
                return_val = u32::MAX;
            }
        }

        return_val
    }
}

fn fit_string_to_11(s: &str) -> [u8; 11] {
    let mut arr = [b' '; 11];

    /*for i in 0..arr.len() {
        s.nth
    }*/

    arr
}

fn flush_ps2_buffer() {
    while inb(0x64) & 1 != 0 {
        let _ = inb(0x60);
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Coordiates {
    x: u16,
    y: u16,
    w: u16,
    h: u16,
}

use alloc::format;
use alloc::string::String;

pub fn format_path_8_3(path: &str) -> String {
    let (dir, filename) = match path.rfind('/') {
        Some(pos) => (&path[..=pos], &path[pos + 1..]),
        None => ("", path),
    };
    if filename.len() == 11 && !filename.contains('.') {
        return String::from(path);
    }
    let (name_part, ext_part) = match filename.rfind('.') {
        Some(dot_idx) => (&filename[..dot_idx], &filename[dot_idx + 1..]),
        None => (filename, ""),
    };
    let mut short_name = String::with_capacity(8);
    for (i, c) in name_part.chars().enumerate() {
        if i >= 8 {
            break;
        }
        short_name.push(c);
    }
    while short_name.len() < 8 {
        short_name.push(' ');
    }
    let mut short_ext = String::with_capacity(3);
    for (i, c) in ext_part.chars().enumerate() {
        if i >= 3 {
            break;
        }
        short_ext.push(c);
    }
    while short_ext.len() < 3 {
        short_ext.push(' ');
    }
    format!("{}{}{}", dir, short_name, short_ext)
}
