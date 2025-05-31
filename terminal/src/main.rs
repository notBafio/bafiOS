#![no_std]
#![no_main]

extern crate alloc;
use kui::widgets::*;
use kui::*;
use core::panic::PanicInfo;
use kui::draw::draw_label;
use alloc::format;

#[global_allocator]
static ALLOC: libk::heap::Allocator = libk::heap::Allocator::new();

use alloc::string::String;
use alloc::borrow::ToOwned;

struct Terminal {
    path: String,
    user: String,
}

static mut TERMINAL: Terminal = Terminal {
    path: String::new(),
    user: String::new(),
};

fn with_terminal<F, R>(f: F) -> R
where
    F: FnOnce(&mut Terminal) -> R,
{
    unsafe { f(&mut (*(&raw mut TERMINAL))) }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    ALLOC.init(0x10_0000);
    ALLOC.first_free.load(core::sync::atomic::Ordering::Relaxed);

    with_terminal(|terminal| {
        terminal.path = String::from("/");
        terminal.user = String::from("guest");
    });

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
        (*(&raw mut draw::KEY_MAP)).add('\n', handle_enter);
    }

    frame.add(tf);
    main.add(frame);
    draw::init(main);

    loop {}
}

pub fn handle_enter(w: &mut Widget) {
    if let Widget::InputLabel(l) = w {
        let prompt = "\n bafiOS@guest> ";

        if l.ch_min as usize >= prompt.len() {
            let cmd = l.label[l.ch_min as usize..].to_owned();

            if cmd.trim().is_empty() {
                l.label.push_str(prompt);
                l.ch_min = l.label.len() as u32;
                draw_label(l);
                return;
            }

            parse_command(w, &cmd);

            if let Widget::InputLabel(l) = w {
                draw_label(l);
            }
        }
    }
}

const MAX_LINES: usize = 12;

fn trim_label_history(label: &mut Label) {
    let mut lines: alloc::vec::Vec<&str> = label.label.split('\n').collect();

    if lines.len() > MAX_LINES {
        let drop_count = lines.len() / 2;
        lines.drain(0..drop_count);
        label.label = lines.join("\n");
        label.ch_min = label.label.len() as u32;
    }
}

pub fn parse_command(w: &mut Widget, command: &str) {
    use alloc::vec::Vec;

    let commands: Vec<&str> = command.split_whitespace().collect();

    if commands.is_empty() {
        return;
    }

    if let Widget::InputLabel(l) = w {
        match commands[0] {
            
            "echo" => {
                if commands.len() > 1 {
                    let output = commands[1..].join(" ");
                    append_output(l, &output);
                } else {
                    append_output(l, "");
                }
            },
            
            "pwd" => {
                let path = String::from(" ") + &with_terminal(|t| t.path.clone());
                append_output(l, &path);
            },
            
            "ls" => {
                let path = with_terminal(|t| t.path.clone());
                let mut output = String::new();

                let entries_count = list_entries(&path);
                for x in 0..entries_count {
                    if let Some(entry) = libk::io::get_entry(&path, x) {
                        if let Ok(name) = core::str::from_utf8(&entry.name) {
                            output.push_str("\n ");
                            output.push_str(name.trim_end());
                        }
                    }
                }

                append_output(l, &output);
            },
            
            "cd" => {
                if commands.len() <= 1 {
                    append_output(l, " Missing directory name");
                    return;
                }

                let target_dir = commands[1];

                with_terminal(|terminal| {
                    if target_dir == ".." {
                        let path = &terminal.path;
                        let search_end = if path.ends_with('/') { path.len() - 1 } else { path.len() };

                        if let Some(last_slash) = path[..search_end].rfind('/') {
                            terminal.path.truncate(last_slash + 1);
                        } else {
                            terminal.path = String::from("/");
                        }

                        append_output(l, "");
                        return;
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
                        if let Some(entry) = libk::io::get_entry(&terminal.path, x) {
                            if let Ok(entry_name) = core::str::from_utf8(&entry.name) {
                                let trimmed_name = entry_name.trim_end();
                                if trimmed_name == target_dir && entry.is_dir() {
                                    dir_exists = true;
                                    break;
                                }
                            }
                        }
                    }

                    if dir_exists {
                        terminal.path = new_path;
                        append_output(l, "");
                    } else {
                        append_output(l, &format!("Directory not found: {}", target_dir));
                    }
                });
            },
            
            "mkdir" => {
                if commands.len() <= 1 {
                    append_output(l, " Missing directory name");
                    return;
                }

                let path = with_terminal(|t| {
                    let mut path = t.path.clone();
                    if !path.ends_with('/') {
                        path.push('/');
                    }
                    path.push_str(commands[1]);
                    path
                });

                libk::io::make_dir(&path);
                append_output(l, &format!(" Created directory: {}", commands[1]));
            },
            
            "mkfile" => {
                if commands.len() <= 1 {
                    append_output(l, " Missing file name");
                    return;
                }

                let path = with_terminal(|t| {
                    let mut path = t.path.clone();
                    if !path.ends_with('/') {
                        path.push('/');
                    }
                    path.push_str(commands[1]);
                    path
                });

                libk::io::make_file(&path);
                append_output(l, &format!(" Created file: {}", commands[1]));
            },
            
            "exec" => {
                if commands.len() <= 1 {
                    append_output(l, " Missing executable name");
                    return;
                }

                let exec_path = with_terminal(|t| {
                    let mut path = t.path.clone();
                    if !path.ends_with('/') {
                        path.push('/');
                    }
                    path.push_str(commands[1]);
                    path
                });

                let default_executor_app = "/USER/EXEC.ELF";

                let exec_path_static = alloc::boxed::Box::leak(exec_path.into_boxed_str());

                let argv: &'static [u32] = alloc::boxed::Box::leak(
                    alloc::vec![
                        exec_path_static.as_ptr() as u32,
                        exec_path_static.len() as u32,
                        0,
                        0,
                    ]
                        .into_boxed_slice(),
                );

                let _ = libk::elf::load_elf(default_executor_app, Some(argv));
            },
            
            "clear" => {
                l.label = String::from("\n bafiOS@guest> ");
                l.ch_min = l.label.len() as u32;
            },
            
            "help" => {
                let help_text = "\n Available commands:\n echo - Display text\n pwd - Print working directory\n ls - List directory contents\n cd - Change directory\n mkdir - Create directory\n mkfile - Create file\n exec - Execute program\n clear - Clear screen\n help - Show this help\n";
                append_output(l, help_text);
            },
            
            _ => {
                append_output(l, &format!(" Unknown command: {}", commands[0]));
            }
        }
    }
}

fn append_output(label: &mut Label, text: &str) {
    trim_label_history(label);

    label.label.push('\n');
    label.label.push_str(text);
    label.label.push_str("\n bafiOS@guest> ");
    label.ch_min = label.label.len() as u32;
}

pub fn list_entries(dir: &str) -> u8 {
    let mut count = 0;

    while count < 255 {
        if libk::io::get_entry(dir, count).is_none() {
            return count;
        }
        count += 1;
    }

    count
}

pub fn expand_path_8_3(path: &str) -> alloc::vec::Vec<u8> {
    if path.len() != 11 || !path.contains(' ') {
        return path.as_bytes().to_vec();
    }

    let name_part = &path[..8];
    let ext_part = &path[8..];

    let trimmed_name = name_part.trim_end();
    let trimmed_ext = ext_part.trim_end();

    let mut result = alloc::vec::Vec::with_capacity(12);

    result.extend_from_slice(trimmed_name.as_bytes());

    if !trimmed_ext.is_empty() {
        result.push(b'.');
        result.extend_from_slice(trimmed_ext.as_bytes());
    }

    result
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}