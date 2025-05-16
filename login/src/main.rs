#![no_std]
#![no_main]

extern crate alloc;
use kui::widgets::*;
use kui::*;
use bafioDb;

use core::panic::PanicInfo;
use kui::widgets::Widget::{InputLabel, Label};

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    libk::println!("LoqsSn");

    let mut lock = Window::new()
        .name("S")
        .width(Size::new("100%"))
        .height(Size::new("100%"))
        .color(Color::rgb(229, 80, 80))
        .action_bar(false)
        .display(Display::None);

    let wp = Widget::Image(
        Image::new("SYS/BG.TGA")
            .height(Size::new("100%"))
            .width(Size::new("100%")),
    );

    let mut frame = Widget::Frame(
        Frame::new()
            .x(Size::new("25%"))
            .y(Size::new("25%"))
            .width(Size::new("50%"))
            .height(Size::new("50%"))
            .color(Color::rgb(255, 130, 130))
            .display(Display::Grid(Grid::new(1, 7))),
    );

    use kui::widgets::Label;

    let place_holder = Widget::Label(Label::new().width(Size::new("0")).height(Size::new("0")));

    let i = Widget::Image(
        Image::new("ICONS/CAT1.TGA")
            .x(Size::new("40%"))
            .width(Size::new("20%"))
            .height(Size::new("100%")),
    );
    let tn = Widget::Label(
        Label::new()
            .text("Username")
            .color(Color::rgb(255, 130, 130))
            .x(Size::new("10%"))
            .width(Size::new("80%"))
            .height(Size::new("30%")),
    );
    let n = Widget::InputLabel(
        Label::new()
            .text("")
            .color(Color::rgb(255, 255, 255))
            .x(Size::new("10%"))
            .width(Size::new("80%"))
            .height(Size::new("80%")),
    );
    let n2 = Widget::InputLabel(
        Label::new()
            .text("")
            .color(Color::rgb(255, 255, 255))
            .x(Size::new("10%"))
            .width(Size::new("80%"))
            .height(Size::new("80%")),
    );
    let tp = Widget::Label(
        Label::new()
            .text("Password")
            .x(Size::new("10%"))
            .color(Color::rgb(255, 130, 130))
            .width(Size::new("80%"))
            .height(Size::new("30%")),
    );
    let btn = Widget::Button(
        Button::new()
            .label("Log in")
            .x(Size::new("33%"))
            .width(Size::new("33%"))
            .height(Size::new("80%"))
            .event(login),
    );

    libk::println!("{:#?}", n);
    libk::println!("{:#?}", n2);

    frame.add(place_holder);
    frame.add(i);
    frame.add(tn);
    frame.add(n);
    frame.add(tp);
    frame.add(n2);
    frame.add(btn);

    lock.add(wp);
    lock.add(frame);
    draw::init(lock);

    libk::println!("Done");

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

pub fn login(w: &mut Widget, a1: u32, _a2: u32, _a3: u32) {
    unsafe {

        match &kui::widgets::WINDOWS[0].children[1] {
            kui::widgets::Widget::Frame(f) => {
                let username = &f.children[3].get_label();
                let password = &f.children[5].get_label();
                let mut p1 = false;
                let mut p2 = false;

                let f = bafioDb::load("/SYS/USERS.DB");
                let uname = f.get("USER");
                let psw = f.get("USERPSW");

                if uname.is_some() && psw.is_some() {
                    match uname.unwrap() {
                        crate::bafioDb::Value::String(s) => {
                            if username.is_some() {
                                if s == username.unwrap() {
                                    p1 = true;
                                }
                            }
                        }
                        _ => {}
                    }

                    match psw.unwrap() {
                        crate::bafioDb::Value::String(s) => {
                            if password.is_some() {
                                if s == core::str::from_utf8_unchecked(&libk::hash::hash_to_hex(&libk::hash::hash_128bit(password.unwrap().as_bytes()))) {
                                    p2 = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }

                if p1 && p2 {
                    libk::println!("SUCCESS");

                    let _ = libk::elf::load_elf("USER/USER.ELF", None);

                    kui::draw::exit(w, kui::widgets::WINDOWS[0].id as u32, 0, 0);
                } else {
                    p1 = false;
                    p2 = false;
                    libk::println!("{}", username.unwrap());
                    libk::println!("{}", password.unwrap());
                }
            },

            _ => {}
        }
    }
}
