#![no_std]
#![no_main]

extern crate alloc;
use alloc::vec::Vec;
use kui::widgets::*;
use kui::*;
use libk::mutex::Mutex;

use bafioDb;

use core::panic::PanicInfo;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    let mut rng = libk::rng::LcgRng::new(core::ptr::addr_of!(ALLOC) as u64);
    let r = rng.range(0, 255) as usize;
    let g = rng.range(0, 255) as usize;
    let b = rng.range(0, 255) as usize;

    let mut main = Window::new()
        .name("Files")
        .width(Size::new("500"))
        .height(Size::new("500"))
        .display(Display::None);

    let folder = "/";
    let entries = list_entries(folder) as usize;
    let rows = 9;
    let cols = 1;

    let mut frame1 = Widget::Frame(
        kui::widgets::Frame::new()
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .color(Color::rgb(18, 52, 88))
            .display(Display::Grid(Grid::new(cols, rows))),
    );

    let mut frame10 = Widget::Frame(
        kui::widgets::Frame::new()
            .width(Size::new("100%"))
            .height(Size::new("100%"))
            .color(Color::rgb(18, 52, 88))
            .display(Display::None),
    );

    let btn_back = Widget::Button(
        Button::new()
            .label("<")
            .color(Color::rgb(61, 139, 221))
            .width(Size::new("20%"))
            .height(Size::new("100%"))
            .event(back)
            .set_args([
                frame1.get_id().unwrap() as u32,
                unsafe { (*(&raw mut kui::widgets::WINDOWS)).len() as u32 },
                0,
            ]),
    );

    let l = Widget::Label(
        Label::new()
            .text(folder)
            .color(Color::rgb(255, 255, 255))
            .x(Size::new("20%"))
            .width(Size::new("80%"))
            .height(Size::new("100%"))
            .text_align(Align::Center),
    );

    frame10.add(btn_back);
    frame10.add(l);
    frame1.add(frame10);

    let f_ile = bafioDb::load("/SYS/ICONS.DB");

    for i in 0..list_entries(folder) {
        let var = libk::io::get_entry(folder, i).unwrap().name;
        let fname = alloc::string::String::from(unsafe { core::str::from_utf8_unchecked(&var) });
        let fake_name =
            unsafe { core::str::from_utf8_unchecked(libk::io::expand_path_8_3(&fname)) };
        let func = alloc::string::String::from(folder) + &fname;

        let color = Color::rgb(255, 255, 255);

        let mut frame2 = Widget::Frame(
            kui::widgets::Frame::new()
                .width(Size::new("100%"))
                .height(Size::new("100%"))
                .color(color)
                .display(Display::Flex),
        );

        let l = Widget::Label(
            Label::new()
                .text(fake_name)
                .color(color)
                .width(Size::new("80%"))
                .height(Size::new("100%"))
                .text_align(Align::Center),
        );

        let mut file_icon = "ICONS/FILE.TGA";
        let f3 = libk::io::File::new(&func);
        if f3.is_dir() {
            file_icon = "ICONS/FOLDER2.TGA";
        } else {
            let icon_path = f_ile.get(f3.get_file_extention());

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

        let mut i = Widget::Image(
            Image::new(file_icon)
                .width(Size::new("32"))
                .height(Size::new("32"))
                .event(executor)
                .set_args([0, 0, unsafe { (*(&raw mut PROGRAMS)).len() as u32 }]),
        );
        if file_icon == "ICONS/FOLDER2.TGA" {
            i = Widget::Image(
                Image::new(file_icon)
                    .width(Size::new("32"))
                    .height(Size::new("32"))
                    .event(new_folder)
                    .set_args([
                        frame1.get_id().unwrap() as u32,
                        unsafe { (*(&raw mut kui::widgets::WINDOWS)).len() as u32 },
                        unsafe { (*(&raw mut PROGRAMS)).len() as u32 },
                    ]),
            );
        }

        unsafe {
            (*(&raw mut PROGRAMS)).push(func);
        }

        frame2.add(i);
        frame2.add(l);
        frame1.add(frame2);
    }

    main.add(frame1);

    kui::draw::init(main);

    loop {}
}

pub fn back(_w: &mut Widget, a1: u32, a2: u32, _a3: u32) {
    let w = unsafe { &mut (*(&raw mut kui::widgets::WINDOWS))[a2 as usize] };

    for f in w.children.iter_mut() {
        if f.get_id().unwrap() == a1 as u16 {
            match f {
                kui::widgets::Widget::Frame(f) => {
                    let dir = &f.children[0];

                    let mut dir_str = alloc::string::String::from("/");

                    match dir {
                        kui::widgets::Widget::Frame(f) => match &f.children[1] {
                            kui::widgets::Widget::Label(l) => {
                                dir_str = alloc::string::String::from(&l.label);
                                dir_str =
                                    alloc::string::String::from(remove_last_component(&dir_str));
                            }
                            _ => {
                                return;
                            }
                        },
                        _ => {
                            return;
                        }
                    }

                    f.children.clear();

                    let mut frame10 = Widget::Frame(
                        kui::widgets::Frame::new()
                            .width(Size::new("100%"))
                            .height(Size::new("100%"))
                            .color(Color::rgb(18, 52, 88))
                            .display(Display::None),
                    );

                    let btn_back = Widget::Button(
                        Button::new()
                            .label("<")
                            .color(Color::rgb(61, 139, 221))
                            .width(Size::new("20%"))
                            .height(Size::new("100%"))
                            .event(back)
                            .set_args([a1, a2, 0]),
                    );

                    let l = Widget::Label(
                        Label::new()
                            .text(&dir_str)
                            .color(Color::rgb(255, 255, 255))
                            .x(Size::new("20%"))
                            .width(Size::new("80%"))
                            .height(Size::new("100%"))
                            .text_align(Align::Center),
                    );

                    frame10.add(btn_back);
                    frame10.add(l);
                    f.add(frame10);

                    let entries = list_entries(&dir_str) as usize;
                    let rows = 8;
                    let cols = (entries + rows - 1) / rows;

                    f.display = kui::widgets::Display::Grid(Grid::new(cols, rows));

                    unsafe { (*(&raw mut PROGRAMS)).clear() }

                    let f_ile = bafioDb::load("/SYS/ICONS.DB");

                    let str_len = list_entries(&dir_str);
                    for i in 0..str_len {
                        let var = libk::io::get_entry(&dir_str, i).unwrap();
                        let fname = alloc::string::String::from(unsafe {
                            core::str::from_utf8_unchecked(&var.name)
                        });
                        let fake_name = unsafe {
                            core::str::from_utf8_unchecked(libk::io::expand_path_8_3(&fname))
                        };
                        let func = alloc::string::String::from(&dir_str) + "/" + &fname;

                        let color = Color::rgb(255, 255, 255);

                        let mut frame2 = Widget::Frame(
                            kui::widgets::Frame::new()
                                .width(Size::new("100%"))
                                .height(Size::new("100%"))
                                .color(color)
                                .display(Display::Flex),
                        );

                        let l = Widget::Label(
                            Label::new()
                                .text(fake_name)
                                .color(color)
                                .width(Size::new("80%"))
                                .height(Size::new("100%"))
                                .text_align(Align::Center),
                        );

                        let mut file_icon = "ICONS/FILE.TGA";
                        let f3 = libk::io::File::new(&func);
                        if f3.is_dir() {
                            file_icon = "ICONS/FOLDER2.TGA";
                        } else {
                            let icon_path = f_ile.get(f3.get_file_extention());

                            if icon_path.is_some() {
                                match icon_path.unwrap() {
                                    crate::bafioDb::Value::String(s) => {
                                        let owned_file_icon = s.clone();
                                        file_icon = alloc::boxed::Box::leak(
                                            owned_file_icon.into_boxed_str(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let mut i = Widget::Image(
                            Image::new(file_icon)
                                .width(Size::new("32"))
                                .height(Size::new("32"))
                                .event(executor)
                                .set_args([0, 0, unsafe { (*(&raw mut PROGRAMS)).len() as u32 }]),
                        );
                        if file_icon == "ICONS/FOLDER2.TGA" {
                            i = Widget::Image(
                                Image::new(file_icon)
                                    .width(Size::new("32"))
                                    .height(Size::new("32"))
                                    .event(new_folder)
                                    .set_args([a1 as u32, a2 as u32, unsafe {
                                        (*(&raw mut PROGRAMS)).len() as u32
                                    }]),
                            );
                        }

                        unsafe {
                            (*(&raw mut PROGRAMS)).push(func);
                        }

                        frame2.add(i);
                        frame2.add(l);
                        f.add(frame2);
                    }
                }
                _ => {}
            }
        }
    }

    kui::draw::draw(w);
    libk::syscall::syscall(41, w.id as u32, 0, 0);
}

pub fn new_folder(w: &mut Widget, a1: u32, a2: u32, a3: u32) {
    let old_value = core::mem::replace(w, kui::widgets::Widget::Label(Label::new()));

    let w = unsafe { &mut (*(&raw mut kui::widgets::WINDOWS))[a2 as usize] };

    for f in w.children.iter_mut() {
        if f.get_id().unwrap() == a1 as u16 {
            match f {
                kui::widgets::Widget::Frame(f) => {
                    f.children.clear();

                    let ndir_str = unsafe { (*(&raw mut PROGRAMS))[a3 as usize].clone() };
                    let dir_str = ndir_str.trim_end();

                    let mut frame10 = Widget::Frame(
                        kui::widgets::Frame::new()
                            .width(Size::new("100%"))
                            .height(Size::new("100%"))
                            .color(Color::rgb(18, 52, 88))
                            .display(Display::None),
                    );

                    let btn_back = Widget::Button(
                        Button::new()
                            .label("<")
                            .color(Color::rgb(61, 139, 221))
                            .width(Size::new("20%"))
                            .height(Size::new("100%"))
                            .event(back)
                            .set_args([a1, a2, 0]),
                    );

                    let l = Widget::Label(
                        Label::new()
                            .text(dir_str)
                            .color(Color::rgb(255, 255, 255))
                            .x(Size::new("20%"))
                            .width(Size::new("80%"))
                            .height(Size::new("100%"))
                            .text_align(Align::Center),
                    );

                    frame10.add(btn_back);
                    frame10.add(l);
                    f.add(frame10);

                    let entries = list_entries(dir_str) as usize;
                    let rows = 9;
                    let cols = 1;

                    f.display = kui::widgets::Display::Grid(Grid::new(cols, rows));

                    unsafe { (*(&raw mut PROGRAMS)).clear() }

                    let f_ile = bafioDb::load("/SYS/ICONS.DB");

                    for i in 0..list_entries(&dir_str) {
                        let var = libk::io::get_entry(&dir_str, i).unwrap().name;
                        let fname = alloc::string::String::from(unsafe {
                            core::str::from_utf8_unchecked(&var)
                        });
                        let fake_name = unsafe {
                            core::str::from_utf8_unchecked(libk::io::expand_path_8_3(&fname))
                        };
                        let func = alloc::string::String::from(dir_str) + "/" + &fname;

                        let color = Color::rgb(255, 255, 255);

                        let mut frame2 = Widget::Frame(
                            kui::widgets::Frame::new()
                                .width(Size::new("100%"))
                                .height(Size::new("100%"))
                                .color(color)
                                .display(Display::Flex),
                        );

                        let l = Widget::Label(
                            Label::new()
                                .text(fake_name)
                                .color(color)
                                .width(Size::new("80%"))
                                .height(Size::new("100%"))
                                .text_align(Align::Center),
                        );

                        let mut file_icon = "ICONS/FILE.TGA";
                        let f3 = libk::io::File::new(&func);
                        if f3.is_dir() {
                            file_icon = "ICONS/FOLDER2.TGA";
                        } else {
                            let icon_path = f_ile.get(f3.get_file_extention());

                            if icon_path.is_some() {
                                match icon_path.unwrap() {
                                    crate::bafioDb::Value::String(s) => {
                                        let owned_file_icon = s.clone();
                                        file_icon = alloc::boxed::Box::leak(
                                            owned_file_icon.into_boxed_str(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }

                        let mut i = Widget::Image(
                            Image::new(file_icon)
                                .width(Size::new("32"))
                                .height(Size::new("32"))
                                .event(executor)
                                .set_args([0, 0, unsafe { (*(&raw mut PROGRAMS)).len() as u32 }]),
                        );
                        if file_icon == "ICONS/FOLDER2.TGA" {
                            i = Widget::Image(
                                Image::new(file_icon)
                                    .width(Size::new("32"))
                                    .height(Size::new("32"))
                                    .event(new_folder)
                                    .set_args([a1 as u32, a2 as u32, unsafe {
                                        (*(&raw mut PROGRAMS)).len() as u32
                                    }]),
                            );
                        }

                        unsafe {
                            (*(&raw mut PROGRAMS)).push(func);
                        }

                        frame2.add(i);
                        frame2.add(l);
                        f.add(frame2);
                    }
                }
                _ => {}
            }
        }
    }

    kui::draw::draw(w);
    libk::syscall::syscall(41, w.id as u32, 0, 0);
}

pub static mut PROGRAMS: Vec<alloc::string::String> = Vec::new();

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

        count += 1;
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn remove_last_component(path: &str) -> &str {
    let trimmed_path = path.trim_end_matches('/');
    if let Some(pos) = trimmed_path.rfind('/') {
        if pos == 0 { "/" } else { &trimmed_path[..pos] }
    } else {
        "/"
    }
}

pub fn executor(_wid: &mut Widget, _a1: u32, _a2: u32, a3: u32) {
    let file2 = bafioDb::load("/SYS/EXEC.DB");

    let func = unsafe { (*(&raw mut PROGRAMS))[a3 as usize].clone() };

    let f3 = libk::io::File::new(&func);
    let default_executor_app = "/USER/DESKTOP/IDE.ELF";

    let exec_path = file2.get(f3.get_file_extention());

    unsafe {
        TEMP_STR = alloc::string::String::from(func.clone());
    }

    if exec_path.is_some() {
        match exec_path.unwrap() {
            crate::bafioDb::Value::String(s) => {
                let owned_exec = s.clone();

                unsafe {
                    let _ = libk::elf::load_elf(
                        &owned_exec,
                        Some(&[
                            (*(&raw const TEMP_STR)).as_ptr() as u32,
                            (*(&raw const TEMP_STR)).len() as u32,
                            0,
                            0,
                        ]),
                    );
                }
            }
            _ => {}
        }
    } else {
        unsafe {
            let _ = libk::elf::load_elf(
                &default_executor_app,
                Some(&[
                    (*(&raw const TEMP_STR)).as_ptr() as u32,
                    (*(&raw const TEMP_STR)).len() as u32,
                    0,
                    0,
                ]),
            );
        }
    }
}

pub static mut TEMP_STR: alloc::string::String = alloc::string::String::new();
