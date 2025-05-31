use crate::widgets::Color;
use crate::widgets::SCREEN;
use core::sync::atomic::AtomicBool;

use crate::widgets::{Align, Button, Display, Frame, Grid, Image, Label, Size, Widget, Window};

use core::sync::atomic::Ordering;

use libk::syscall;

pub static FLAG: AtomicBool = AtomicBool::new(false);

pub static mut FRAMEBUFFER: u32 = 0;
pub static mut W_WIDTH: u32 = 0;
pub static mut W_HEIGHT: u32 = 0;
pub static mut INPUT: (u16, u16) = (0, 0);
pub static mut EXITING: AtomicBool = AtomicBool::new(false);

pub fn write_pixel(row: u32, col: u32, color: Color) {
    let width = unsafe { W_WIDTH };
    let height = unsafe { W_HEIGHT };
    let depth = unsafe { (*(&raw mut SCREEN)).depth };
    let framebuffer = unsafe { FRAMEBUFFER };

    if framebuffer == 0 || width == 0 || height == 0 {
        return;
    }

    if col < width && row < height {
        unsafe {
            match depth {

                16 => {
                    *((framebuffer as *mut u16).add((row * width + col) as usize)) = color.to_u16();
                }

                32 => {
                    *((framebuffer as *mut u32).add((row * width + col) as usize)) = color.to_u32();
                }

                24 => {
                    let c = color.to_u24();
                    *((framebuffer as *mut u8).add(((row * width + col) * 3 + 0) as usize)) = c[0];
                    *((framebuffer as *mut u8).add(((row * width + col) * 3 + 1) as usize)) = c[1];
                    *((framebuffer as *mut u8).add(((row * width + col) * 3 + 2) as usize)) = c[2];
                }

                _ => {
                    libk::println!("Unsupported color depth: {}", depth);
                }
            }
        }
    }
}

pub fn write_pixel_u16(row: u32, col: u32, color: u16) {
    let width = unsafe { W_WIDTH };
    let height = unsafe { W_HEIGHT };
    let framebuffer = unsafe { FRAMEBUFFER };

    if col < width && row < height {
        unsafe {
            *((framebuffer as *mut u16).add((row * width + col) as usize)) = color;
        }
    }
}

pub fn write_pixel_u32(row: u32, col: u32, color: u32) {
    let width = unsafe { W_WIDTH };
    let height = unsafe { W_HEIGHT };
    let framebuffer = unsafe { FRAMEBUFFER };

    if col < width && row < height {
        unsafe {
            *((framebuffer as *mut u32).add((row * width + col) as usize)) = color;
        }
    }
}

pub fn write_pixel_u24(row: u32, col: u32, rgb: [u8; 3]) {
    let width  = unsafe { W_WIDTH };
    let height = unsafe { W_HEIGHT };
    let framebuffer = unsafe { FRAMEBUFFER };

    if row < height && col < width {
        let pixel_index = (row as usize * width as usize + col as usize) * 3;

        unsafe {
            let p = (framebuffer as *mut u8).add(pixel_index);
            *p = rgb[0];
            *p.add(1) = rgb[1];
            *p.add(2) = rgb[2];
        }
    }
}


pub fn draw_rectangle(h_start: u32, h_end: u32, w_start: u32, w_end: u32, color: Color) {
    let width = unsafe { W_WIDTH };
    let height = unsafe { W_HEIGHT };

    if h_start >= height || w_start >= width {
        return;
    }

    let h_end_clamped = h_end.min(height);
    let w_end_clamped = w_end.min(width);

    if h_end_clamped <= h_start || w_end_clamped <= w_start {
        return;
    }

    let depth = unsafe { (*(&raw mut SCREEN)).depth };

    match depth {

        16 => {
            let color = color.to_u16();

            for j in h_start..h_end_clamped {
                for i in w_start..w_end_clamped {
                    write_pixel_u16(j, i, color)
                }
            }
        }

        32 => {
            let color = color.to_u32();

            for j in h_start..h_end_clamped {
                for i in w_start..w_end_clamped {
                    write_pixel_u32(j, i, color)
                }
            }
        }

        24 => {
            let color = color.to_u24();

            for j in h_start..h_end_clamped {
                for i in w_start..w_end_clamped {
                    write_pixel_u24(j, i, color)
                }
            }
        }


        _ => {
            libk::println!("Unsupported color depth: {}", depth);
        }
    }
}

pub fn draw(w: &mut Window) {
    unsafe { FRAMEBUFFER = w.buffer };
    unsafe {
        W_WIDTH = w.width.absolute.unwrap();
    }
    unsafe {
        W_HEIGHT = w.height.absolute.unwrap();
    }

    draw_window(w);

    if w.action_bar {
        draw_rectangle(
            0,
            25,
            0,
            w.width.absolute.unwrap(),
            Color::rgb(251, 119, 60),
        );
        draw_string(
            &w.name,
            w.text_color,
            10,
            12,
            w.width.absolute.unwrap() as usize - 25,
            20,
        );
    }

    match w.display {
        Display::Flex => mk_flex_w(w),
        Display::Grid(grid) => mk_grid_w(w, grid),

        _ => {}
    }

    if w.action_bar {
        for i in 0..w.children.len() {
            recursive_draw(
                &mut w.children[i],
                0,
                25,
                w.width.absolute.unwrap(),
                w.height.absolute.unwrap() - 25,
                w.display,
            );
        }
    } else {
        for i in 0..w.children.len() {
            recursive_draw(
                &mut w.children[i],
                0,
                0,
                w.width.absolute.unwrap(),
                w.height.absolute.unwrap(),
                w.display,
            );
        }
    }
}

pub fn draw_handler(id: u16, _dw: i16, _dh: i16) {
    while FLAG
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {}

    unsafe {
        for i in 0..16 {
            if crate::widgets::WINDOWS[i].id == id && id != 0 {
                draw(&mut crate::widgets::WINDOWS[i]);
            }
        }
    }

    FLAG.store(false, Ordering::Relaxed);
}

pub fn cap(n: i16) -> u32 {
    if n < 0 { 0 } else { n as u32 }
}

pub fn init(mut w: Window) {
    unsafe {
        (*(&raw mut crate::widgets::SCREEN)).init();

        let add_w_result = *(libk::syscall::add_window(w.to_window()) as *const (u32, u32));
        w.id = add_w_result.0 as u16;
        w.buffer = add_w_result.1;

        if w.action_bar == true {
            let exit_btn = Widget::Button(Button {
                id: 0,
                label: crate::alloc::string::String::from("x"),
                x: Size::from_u32(0),
                y: Size::from_u32(0),
                width: Size::new("19"),
                height: Size::new("19"),
                color: Color::rgb(245, 0, 79),
                event: crate::draw::exit,
                padding: Size::new("0"),
                border_radius: Size::new("0"),
                text_color: Color::rgb(0, 0, 0),
                real_x: 0,
                real_y: 0,
                text_align: Align::Center,
                args: [w.id as u32, 0, 0],
            });

            w.add_exit(exit_btn);
        }

        (*(&raw mut crate::widgets::WINDOWS)).push(w.clone());

        let idx = (*(&raw mut crate::widgets::WINDOWS)).len() - 1;

        draw(&mut crate::widgets::WINDOWS[idx]);

        let wi = crate::widgets::WINDOWS[idx].width.absolute.unwrap() as u16;
        let h = crate::widgets::WINDOWS[idx].height.absolute.unwrap() as u16;
        let x = crate::widgets::WINDOWS[idx].x.absolute.unwrap() as u16;
        let y = crate::widgets::WINDOWS[idx].y.absolute.unwrap() as u16;

        let c: syscall::Coordiates = syscall::Coordiates {
            w: wi,
            h: h,
            x: x,
            y: y,
        };

        syscall::write_to_screen(w.buffer, c);
    }
}

pub fn recursive_draw(head: &mut Widget, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
    match head {
        Widget::Frame(value) => {
            value.reload(px, py, pw, ph, display);
            draw_frame(value);

            match value.display {
                Display::Flex => mk_flex(value),
                Display::Grid(grid) => mk_grid(value, grid),

                _ => {}
            }

            for i in 0..value.children.len() {
                recursive_draw(
                    &mut value.children[i],
                    value.real_x,
                    value.real_y,
                    value.width.absolute.unwrap(),
                    value.height.absolute.unwrap(),
                    value.display,
                );
            }
        }

        Widget::Button(value) => {
            value.reload(px, py, pw, ph, display);
            draw_button(value);
        }
        Widget::Label(value) => {
            value.reload(px, py, pw, ph, display);
            draw_label(value);
        }

        Widget::InputLabel(value) => {
            value.reload(px, py, pw, ph, display);
            draw_label(value);
            if unsafe { !KB } {
                syscall::add_task(keyboard_thread as u32, None);
                unsafe {
                    KB = true;
                }
            }
        }

        Widget::Image(value) => {
            value.reload(px, py, pw, ph, display);
            draw_image(value);
        }
        _ => {}
    }
}

pub fn mk_grid(frame: &mut Frame, grid: Grid) {
    let cell_width =
        crate::kui_ceil(frame.width.absolute.unwrap() as f32 / grid.columns as f32) as u32;
    let cell_height =
        crate::kui_ceil(frame.height.absolute.unwrap() as f32 / grid.rows as f32) as u32;

    let mut x = 0;
    let mut y = 0;

    for (i, child) in frame.children.iter_mut().enumerate() {

        let rel_x = if child.get_x().relative.is_none() {
            0.0
        } else {
            frame.width.absolute.unwrap() as f32 / 100.0 * child.get_x().relative.unwrap() as f32
        };

        libk::println!("{:?}<", rel_x);

        child.set_x(frame.real_x + x + crate::kui_ceil(rel_x) as u32);
        child.set_y(frame.real_y + y);

        if let Some(relative_height) = child.get_height().relative {
            child.set_height(Size {
                absolute: Some(
                    crate::kui_ceil(cell_height as f32 / 100.0 * relative_height as f32) as u32,
                ),
                relative: Some(relative_height),
            });
        }

        if let Some(relative_width) = child.get_width().relative {
            child.set_width(Size {
                absolute: Some(
                    crate::kui_ceil(cell_width as f32 / 100.0 * relative_width as f32) as u32,
                ),
                relative: Some(relative_width),
            });
        }

        x += cell_width;

        if (i + 1) % grid.columns == 0 {
            y += cell_height;
            x = 0;
        }
    }
}

pub fn mk_flex(frame: &mut Frame) {
    let mut flex_square_width = 0;

    for child in frame.children.iter_mut() {
        child.reload(
            frame.real_x,
            frame.real_y,
            frame.width.absolute.unwrap(),
            frame.height.absolute.unwrap(),
            Display::None,
        );
        flex_square_width += child.width() + child.padding() * 2;
    }

    let base_x = crate::kui_ceil(
        frame.real_x as f32
            + (frame.width.absolute.unwrap() as f32 - flex_square_width as f32) / 2.0,
    ) as u32;
    let mut offset = 0;

    for child in frame.children.iter_mut() {
        child.set_x(base_x + offset);
        child.set_y(frame.real_y + (frame.height.absolute.unwrap() - child.height()) / 2);

        offset += child.width() + child.padding() * 2;
    }
}

pub fn mk_flex_w(window: &mut Window) {
    let mut flex_square_width = 0;

    for child in window.children.iter_mut() {
        child.reload(
            0,
            0,
            window.width.absolute.unwrap(),
            window.height.absolute.unwrap(),
            Display::None,
        );
        flex_square_width += child.width() + child.padding() * 2;
    }

    let base_x =
        crate::kui_ceil((window.width.absolute.unwrap() as f32 - flex_square_width as f32) / 2.0)
            as u32;
    let mut offset = 0;

    for child in window.children.iter_mut() {
        child.set_x(base_x + offset);
        child.set_y((window.height.absolute.unwrap() - child.height()) / 2);

        offset += child.width() + child.padding() * 2;
    }
}

pub fn mk_grid_w(window: &mut Window, grid: Grid) {
    let cell_width = window.width.absolute.unwrap() / grid.columns as u32;
    let cell_height = window.height.absolute.unwrap() / grid.rows as u32;

    let mut x = 0;
    let mut y = 0;

    for (i, child) in window.children.iter_mut().enumerate() {
        child.set_x(x);
        child.set_y(y);

        match child.get_height().relative {
            None => {}
            _ => {
                child.set_height(Size {
                    absolute: Some(crate::kui_ceil(
                        cell_height as f32 / 100.0 * child.get_height().relative.unwrap() as f32,
                    ) as u32),
                    relative: child.get_height().relative,
                });
            }
        }

        match child.get_width().relative {
            None => {}
            _ => {
                child.set_width(Size {
                    absolute: Some(crate::kui_ceil(
                        cell_width as f32 / 100.0 * child.get_width().relative.unwrap() as f32,
                    ) as u32),
                    relative: child.get_width().relative,
                });
            }
        }

        x += cell_width;

        if (i + 1) % grid.columns == 0 {
            y += cell_height;
            x = 0;
        }
    }
}

pub fn draw_window(w: &mut Window) {
    unsafe { FRAMEBUFFER = w.buffer };
    unsafe {
        W_WIDTH = w.width.absolute.unwrap();
    }
    unsafe {
        W_HEIGHT = w.height.absolute.unwrap();
    }

    let mut h_start = 0;

    if w.action_bar == true {
        h_start = 0;
    }

    let h_end = h_start + w.height.absolute.unwrap();
    let w_start = 0;
    let w_end = w_start + w.width.absolute.unwrap();

    draw_rectangle(h_start, h_end, w_start, w_end, w.color);
}

pub fn draw_button(item: &Button) {
    let h_start = item.real_y + item.padding.absolute.unwrap();
    let w_start = item.real_x + item.padding.absolute.unwrap();

    let h_end = h_start + item.height.absolute.unwrap();
    let w_end = w_start + item.width.absolute.unwrap();

    draw_rectangle(h_start, h_end, w_start, w_end, item.color);
    match item.text_align {
        Align::Center => {
            draw_string_centered(
                &item.label,
                item.text_color,
                w_start as usize,
                h_start as usize,
                item.width.absolute.unwrap() as usize,
                item.height.absolute.unwrap() as usize,
            );
        }
        Align::Left => {
            draw_string(
                &item.label,
                item.text_color,
                w_start as usize,
                h_start as usize,
                item.width.absolute.unwrap() as usize,
                item.height.absolute.unwrap() as usize,
            );
        }
    }
}

pub fn draw_label(item: &Label) {
    let h_start = item.real_y + item.padding.absolute.unwrap();
    let w_start = item.real_x + item.padding.absolute.unwrap();

    let h_end = h_start + item.height.absolute.unwrap();
    let w_end = w_start + item.width.absolute.unwrap();

    draw_rectangle(h_start, h_end, w_start, w_end, item.color);
    match item.text_align {
        Align::Center => {
            draw_string_centered(
                &item.label,
                item.text_color,
                w_start as usize,
                h_start as usize,
                item.width.absolute.unwrap() as usize,
                item.height.absolute.unwrap() as usize,
            );
        }
        Align::Left => {
            draw_string(
                &item.label,
                item.text_color,
                w_start as usize,
                h_start as usize,
                item.width.absolute.unwrap() as usize,
                item.height.absolute.unwrap() as usize,
            );
        }
    }
}

pub fn draw_frame(item: &Frame) {
    let h_start = item.real_y + item.padding.absolute.unwrap();
    let w_start = item.real_x + item.padding.absolute.unwrap();

    let h_end = h_start + item.height.absolute.unwrap();
    let w_end = w_start + item.width.absolute.unwrap();

    draw_rectangle(h_start, h_end, w_start, w_end, item.color);
}

pub fn draw_image(item: &Image) {
    match item.header.bpp {
        32 => r_n_n_32(
            (item.ptr + 18) as *const u8,
            item.header.width as u32,
            item.header.height as u32,
            item.width.absolute.unwrap(),
            item.height.absolute.unwrap(),
            item.real_x,
            item.real_y,
        ),
        24 => r_n_n_24(
            (item.ptr + 18) as *const u8,
            item.header.width as u32,
            item.header.height as u32,
            item.width.absolute.unwrap(),
            item.height.absolute.unwrap(),
            item.real_x,
            item.real_y,
        ),
        _ => {}
    }
}

pub fn r_n_n_32(
    src_bitmap_ptr: *const u8,
    src_width: u32,
    src_height: u32,
    dest_width: u32,
    dest_height: u32,
    rx: u32,
    ry: u32,
) {
    let x_ratio = src_width as f32 / dest_width as f32;
    let y_ratio = src_height as f32 / dest_height as f32;

    for y in 0..dest_height {
        for x in 0..dest_width {
            let src_x = (x as f32 * x_ratio) as u32;
            let src_y = (y as f32 * y_ratio) as u32;

            let src_index = (src_y * src_width + src_x) as usize * 4;

            let (r, g, b, a) = unsafe {
                let byte_ptr = src_bitmap_ptr.add(src_index);

                let b = *byte_ptr as usize;
                let g = *byte_ptr.add(1) as usize;
                let r = *byte_ptr.add(2) as usize;
                let a = *byte_ptr.add(3) as usize;
                (r, g, b, a)
            };

            if a != 0 {
                write_pixel(ry + y, rx + x, Color::rgba(r as u8, g as u8, b as u8, a as u8));
            }
        }
    }
}

pub fn r_n_n_24(
    src_bitmap_ptr: *const u8,
    src_width: u32,
    src_height: u32,
    dest_width: u32,
    dest_height: u32,
    rx: u32,
    ry: u32,
) {
    let x_ratio = src_width as f32 / dest_width as f32;
    let y_ratio = src_height as f32 / dest_height as f32;
    let src_stride = (src_width * 3) as usize;

    for y in 0..dest_height {
        for x in 0..dest_width {
            let src_x = (x as f32 * x_ratio) as u32;
            let src_y = (y as f32 * y_ratio) as u32;

            let src_index = (src_y * src_stride as u32 + src_x * 3) as usize;

            let b = unsafe { *src_bitmap_ptr.add(src_index) };
            let g = unsafe { *src_bitmap_ptr.add(src_index + 1) };
            let r = unsafe { *src_bitmap_ptr.add(src_index + 2) };

            let color = Color::rgb(r, g, b);

            write_pixel(ry + y, rx + x, color);
        }
    }
}

pub fn draw_string(
    string: &str,
    color: Color,
    container_x: usize,
    container_y: usize,
    container_width: usize,
    container_height: usize,
) {
    let mut x = container_x;
    let mut y = container_y;

    let line_height = unsafe { (*(&raw mut crate::psf::FONT)).get_char('F').height };

    for ch in string.chars() {
        if ch == '\n' {
            x = container_x;
            y += line_height as usize;
            continue;
        }
        let glyph = unsafe { (*(&raw mut crate::psf::FONT)).get_char(ch) };

        let glyph_width = glyph.width as usize;
        let glyph_height = glyph.height as usize;

        if x + glyph_width > container_x + container_width {
            x = container_x;
            y += line_height as usize;
        }
        if y + glyph_height > container_y + container_height {
            break;
        }
        for i in 0..glyph_height {
            for j in 0..glyph_width {
                let screen_x = x + j;
                let screen_y = y + i;
                if screen_x < container_x + container_width
                    && screen_y < container_y + container_height
                {
                    if glyph.map[i * glyph_width + j] {
                        write_pixel(screen_y as u32, screen_x as u32, color);
                    }
                }
            }
        }
        x += glyph_width;
    }
}

pub fn draw_string_centered(
    string: &str,
    color: Color,
    container_x: usize,
    container_y: usize,
    container_width: usize,
    container_height: usize,
) {
    let mut total_width = 0;
    let mut max_glyph_height = 0;
    let mut lines: alloc::vec::Vec<&str> = alloc::vec::Vec::new();

    for line in string.split('\n') {
        lines.push(line);

        let mut line_width = 0;
        let mut line_max_height = 0;

        for char in line.chars() {
            let glyph = unsafe { (*(&raw mut crate::psf::FONT)).get_char(char) };
            line_width += glyph.width;

            if glyph.height > line_max_height {
                line_max_height = glyph.height;
            }
        }

        if line_width > total_width {
            total_width = line_width;
        }

        if line_max_height > max_glyph_height {
            max_glyph_height = line_max_height;
        }
    }

    let total_height = max_glyph_height as usize * lines.len();
    let start_y = container_y + (container_height - total_height) / 2;
    let mut y = start_y;

    for line in lines {
        let mut line_width = 0;

        for char in line.chars() {
            let glyph = unsafe { (*(&raw mut crate::psf::FONT)).get_char(char) };
            line_width += glyph.width;
        }

        let start_x = container_x + (container_width - line_width as usize) / 2;
        let mut x = start_x;

        for char in line.chars() {
            let glyph = unsafe { (*(&raw mut crate::psf::FONT)).get_char(char) };

            let glyph_width = glyph.width;
            let glyph_height = glyph.height;

            for i in 0..glyph_height as usize {
                for j in 0..glyph_width as usize {
                    let screen_x = x + j;
                    let screen_y = y + i;

                    if screen_x < container_x + container_width
                        && screen_y < container_y + container_height
                    {
                        if glyph.map[(i * glyph_width as usize + j) as usize] {
                            write_pixel(screen_y as u32, screen_x as u32, color);
                        }
                    }
                }
            }

            x += glyph_width as usize;
        }

        y += max_glyph_height as usize;
    }
}

pub fn mouse_handler(wid: u32, mx: u32, my: u32) -> ! {
    unsafe {
        INPUT.0 = wid as u16;

        if (*(&raw mut EXITING))
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            syscall::exit();
        }

        let windows = &mut (*(&raw mut crate::widgets::WINDOWS));

        if let Some(window) = windows.iter_mut().find(|w| w.id == INPUT.0 as u16) {
            for child in &mut window.children {
                recursive_check(child, mx, my, wid as u16);
            }
        }
    }

    syscall::exit();
}

pub fn recursive_check(head: &mut Widget, x: u32, y: u32, wid: u16) {
    match head {
        Widget::Frame(value) => {
            for i in 0..value.children.len() {
                recursive_check(&mut value.children[i], x, y, wid);
            }
        }

        Widget::Button(_) => {
            check_widget(head, x, y, wid);
        }
        Widget::Label(_) => {
            check_widget(head, x, y, wid);
        }
        Widget::InputLabel(_) => {
            check_widget(head, x, y, wid);
        }
        Widget::Image(_) => {
            check_widget(head, x, y, wid);
        }
        _ => {}
    }
}

pub fn check_widget(w: &mut Widget, x: u32, y: u32, wid: u16) {
    let wx = w.x();
    let wy = w.y();
    let ww = w.get_width().absolute.unwrap();
    let wh = w.get_height().absolute.unwrap();

    if x >= wx && x <= (wx + ww) && y >= wy && y <= (wy + wh) {
        match w {
            Widget::Button(b) => {
                let a1 = b.args[0];
                let a2 = b.args[1];
                let a3 = b.args[2];

                let e = w.get_event();
                unsafe {
                    INPUT.1 = 0;
                }

                e(w, a1, a2, a3);
            }

            Widget::InputLabel(l) => unsafe {
                let is_different_field = INPUT.1 != l.id;

                INPUT.1 = l.id;

                if is_different_field {
                    CHAR_BUFFER = [0; 64];
                }
            },

            Widget::Image(i) => {
                let a1 = i.args[0];
                let a2 = i.args[1];
                let a3 = i.args[2];

                let e = w.get_event();
                unsafe {
                    INPUT.1 = 0;
                }

                e(w, a1, a2, a3);
            }

            _ => unsafe { INPUT.1 = 0 },
        }
    }
}

pub static mut CHAR_BUFFER: [u8; 64] = [0; 64];
pub static mut KB: bool = false;

#[unsafe(no_mangle)]
#[inline(never)]
pub extern "C" fn keyboard_thread() {

    let mut c = 0;
    let mut changed = false;
    loop {
        if unsafe { KB } == false {
            break;
        }

        unsafe {
            c += 1;

            if c >= 100_000 {
                for i in 0..64 {
                    let byte = core::ptr::read_volatile(&CHAR_BUFFER[i]);
                    if byte == 0 {
                        break;
                    }

                    while FLAG
                        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                        .is_err()
                    {}

                    changed = true;
                    find_input_widget(byte as char);
                    core::ptr::write_volatile(&mut CHAR_BUFFER[i], 0);

                    FLAG.store(false, Ordering::Release);
                }

                if changed {
                    changed = false;
                    syscall::syscall(41, INPUT.0 as u32, 0, 0);
                }

                c = 0;
            }
        }
    }

    syscall::exit();
}

pub fn find_input_widget(char: char) {
    unsafe {
        if let Some(window) = (*(&raw mut crate::widgets::WINDOWS))
            .iter_mut()
            .find(|w| w.id == INPUT.0 as u16)
        {
            for child in &mut window.children {
                recursive_input_check(child, char);
            }
        }
    }
}

pub fn recursive_input_check(head: &mut Widget, char: char) {
    match head {
        Widget::InputLabel(l) => {
            if unsafe { INPUT.1 } == l.id {
                if unsafe { (*(&raw mut KEY_MAP)).get_event(char).is_some() } {
                    let event = unsafe { (*(&raw mut KEY_MAP)).get_event(char).unwrap() };
                    event(head);
                    return;
                }

                match char {
                    '\x08' => {
                        if l.label.len() > l.ch_min as usize {
                            l.label.pop();
                            draw_label(l);
                        }
                    }
                    '\x02' => {}
                    '\n' => {
                        if l.label.len() < l.ch_max as usize {
                            l.label.push(char);
                            draw_label(l);
                        }
                    }
                    _ => {
                        if l.label.len() < l.ch_max as usize {
                            l.label.push(char);
                            draw_label(l);
                        }
                    }
                }
            }
        }

        Widget::Frame(value) => {
            for i in 0..value.children.len() {
                recursive_input_check(&mut value.children[i], char);
            }
        }
        _ => {}
    }
}

pub fn dealloc_check(head: &Widget) {
    match head {
        Widget::Frame(value) => {
            for i in 0..value.children.len() {
                dealloc_check(&value.children[i]);
            }
        }

        Widget::Image(value) => {
            syscall::free(value.ptr);
        }
        _ => {}
    }
}

pub fn exit(_w: &mut Widget, arg1: u32, arg2: u32, arg3: u32) {

    unsafe {
        KB = false;
    }

    unsafe { (*(&raw mut EXITING)).store(true, Ordering::Relaxed) };

    unsafe {
        if let Some(window) = (*(&raw mut crate::widgets::WINDOWS)).iter_mut().find(|w| w.id as u32 == arg1) {
            (*(&raw mut crate::psf::FONT)).unload();

            for child in &mut window.children {
                dealloc_check(child);
            }

            syscall::remove_window(arg1);
        }

        for i in 0..(*(&raw mut crate::widgets::WINDOWS)).len() {
            if (*(&raw mut crate::widgets::WINDOWS))[i].id == arg1 as u16 {
                (*(&raw mut crate::widgets::WINDOWS)).remove(i);
                break;
            }
        }
    }

    libk::syscall::exit();
}

pub fn resize_handler(id: u32, w: u32, h: u32, buffer: u32) -> ! {

    while FLAG
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        .is_err()
    {}

    unsafe {
        if buffer == 0 {
            FLAG.store(false, Ordering::Relaxed);
            syscall::exit();
        }

        for i in 0..(*(&raw mut crate::widgets::WINDOWS)).len() {
            if crate::widgets::WINDOWS[i].id == id as u16 {
                let old_buffer = crate::widgets::WINDOWS[i].buffer;

                crate::widgets::WINDOWS[i].width = Size::from_u32(w);
                crate::widgets::WINDOWS[i].height = Size::from_u32(h);
                crate::widgets::WINDOWS[i].buffer = buffer;
                FRAMEBUFFER = buffer;
                W_WIDTH = w;
                W_HEIGHT = h;
                draw(&mut crate::widgets::WINDOWS[i]);

                syscall::write_wid_to_screen(crate::widgets::WINDOWS[i].id as u32);
                if old_buffer != 0 && old_buffer != crate::widgets::WINDOWS[i].buffer {
                    syscall::free(old_buffer);
                }

                break;
            } else {
            }
        }
    }

    FLAG.store(false, Ordering::Relaxed);

    syscall::exit();
}

pub static mut KEY_MAP: CustomKeys = CustomKeys {
    key: ['\0'; 64],
    event: [|_| {}; 64],
    count: 0,
};

#[derive(Copy, Clone, Debug)]
pub struct CustomKeys {
    pub key: [char; 64],
    pub event: [fn(&mut Widget); 64],
    pub count: usize,
}

impl CustomKeys {
    pub fn add(&mut self, char: char, event: fn(&mut Widget)) {
        if self.count < 64 {
            self.key[self.count] = char;
            self.event[self.count] = event;

            self.count += 1;
        }
    }

    pub fn remove(&mut self, char: char) {
        for i in 0..self.count {
            if self.key[i] == char {
                if i < self.count - 1 {
                    self.key[i] = self.key[self.count - 1];
                    self.event[i] = self.event[self.count - 1];
                }

                self.key[self.count - 1] = '\0';
                self.event[self.count - 1] = |_| {};
                self.count -= 1;

                break;
            }
        }
    }

    pub fn get_event(&self, char: char) -> Option<fn(&mut Widget)> {
        for i in 0..self.count {
            if self.key[i] == char {
                return Some(self.event[i]);
            }
        }

        None
    }
}
