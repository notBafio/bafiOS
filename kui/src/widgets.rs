use alloc::boxed::Box;
use alloc::fmt::Debug;
use alloc::string::String;
use alloc::vec::Vec;
use libk::io::*;
use libk::syscall;
use libk::syscall::Items;

pub static mut WINDOWS: Vec<Window> = Vec::new();

pub static mut SCREEN: ScreenStats = ScreenStats {
    depth: 8,
    real_x: 0,
    real_y: 0,
    width: Size {
        absolute: Some(320),
        relative: None,
    },
    height: Size {
        absolute: Some(200),
        relative: None,
    },
};

pub static NULL_SIZE: Size = Size {
    absolute: Some(0),
    relative: None,
};

pub static NULL_WINDOW: Window = Window {
    id: 0,
    name: String::new(),
    x: NULL_SIZE,
    y: NULL_SIZE,
    width: NULL_SIZE,
    height: NULL_SIZE,
    border_radius: 0,
    color: Color::new(),
    children: Vec::new(),
    parent: ScreenStats {
        depth: 0,
        height: NULL_SIZE,
        width: NULL_SIZE,
        real_x: 0,
        real_y: 0,
    },
    text_color: Color::new(),
    display: Display::None,
    buffer: 0,
    action_bar: true,
    wtype: Items::Window,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Align {
    Center,
    Left,
}

#[derive(Debug, Copy, Clone)]
pub struct ScreenStats {
    pub depth: usize,
    pub height: Size,
    pub width: Size,
    pub real_x: u32,
    pub real_y: u32,
}

impl ScreenStats {
    pub fn init(&mut self) {
        let boot = syscall::syscall(0, 0, 0, 0);
        let info = unsafe { *(boot as *const libk::boot::BootInfo) };

        self.depth = info.mode.bpp as usize;
        self.width = Size::from_u16(info.mode.width);
        self.height = Size::from_u16(info.mode.height);
    }
}

/*#[derive(Debug, Copy, Clone)]
pub enum Color {
    U8(u8),
    U16(u16),
    U24([u8; 3]),
    U32(u32),
    U64(u64),
}

impl Color {
    pub fn rgb(r: usize, g: usize, b: usize) -> Color {
        match unsafe { SCREEN.depth } {
            8 => Color::U8(Color::color_to_u8(r, g, b)),
            16 => Color::U16(Color::color_to_u16(r, g, b)),
            24 => Color::U24([r as u8, g as u8, b as u8]),
            32 => Color::U32(Color::color_to_u32(r, g, b, 0xFF)),
            64 => Color::U64(Color::color_to_u64(r, g, b, 0xFF)),

            _ => Color::U8(0x00),
        }
    }

    pub fn rgba(r: usize, g: usize, b: usize, a: usize) -> Color {
        match unsafe { SCREEN.depth } {
            8 => Color::U8(Color::color_to_u8(r, g, b)),
            16 => Color::U16(Color::color_to_u16(r, g, b)),
            24 => Color::U24([r as u8, g as u8, b as u8]),
            32 => Color::U32(Color::color_to_u32(r, g, b, a)),
            64 => Color::U64(Color::color_to_u64(r, g, b, a)),

            _ => Color::U8(0x00),
        }
    }

    fn color_to_u8(red: usize, green: usize, blue: usize) -> u8 {
        let red = (red & 0xFF) as u8;
        let green = (green & 0xFF) as u8;
        let blue = (blue & 0xFF) as u8;

        (red >> 5 << 5) | (green >> 5 << 2) | (blue >> 6)
    }

    fn color_to_u16(red: usize, green: usize, blue: usize) -> u16 {
        let red = (red & 0xFF) as u16;
        let green = (green & 0xFF) as u16;
        let blue = (blue & 0xFF) as u16;

        (red >> 3 << 11) | (green >> 2 << 5) | (blue >> 3)
    }

    fn color_to_u32(red: usize, green: usize, blue: usize, alpha: usize) -> u32 {
        let red = (red & 0xFF) as u32;
        let green = (green & 0xFF) as u32;
        let blue = (blue & 0xFF) as u32;
        let alpha = (alpha & 0xFF) as u32;

        (alpha << 24) | (red << 16) | (green << 8) | blue
    }

    fn color_to_u64(red: usize, green: usize, blue: usize, alpha: usize) -> u64 {
        let red = (red & 0xFF) as u64;
        let green = (green & 0xFF) as u64;
        let blue = (blue & 0xFF) as u64;
        let alpha = (alpha & 0xFF) as u64;

        (alpha << 48) | (red << 32) | (green << 16) | blue
    }

    pub fn as_u8(self) -> u8 {
        match self {
            Color::U8(value) => value,
            Color::U16(value) => value as u8,
            Color::U32(value) => value as u8,
            Color::U64(value) => value as u8,
        }
    }

    pub fn as_u16(self) -> u16 {
        match self {
            Color::U8(value) => value as u16,
            Color::U16(value) => value as u16,
            Color::U32(value) => value as u16,
            Color::U64(value) => value as u16,
        }
    }

    pub fn as_u32(self) -> u32 {
        match self {
            Color::U8(value) => value as u32,
            Color::U16(value) => value as u32,
            Color::U32(value) => value as u32,
            Color::U64(value) => value as u32,
        }
    }

    pub fn as_u64(self) -> u64 {
        match self {
            Color::U8(value) => value as u64,
            Color::U16(value) => value as u64,
            Color::U32(value) => value as u64,
            Color::U64(value) => value as u64,
        }
    }
}*/

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {

    pub const fn new() -> Self {
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color { r, g, b, a: 0 }
    }

    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
        Color { r, g, b, a }
    }

    pub fn to_u16(&self) -> u16 {
        let r = (self.r >> 3) as u16;
        let g = (self.g >> 2) as u16;
        let b = (self.b >> 3) as u16;
        (r << 11) | (g << 5) | b
    }

    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn to_u24(&self) -> [u8; 3] {
        [self.b, self.g, self.r]
    }

    pub fn from_u16(rgb: u16) -> Self {
        let r5 = ((rgb >> 11) & 0x1F) as u8;
        let g6 = ((rgb >> 5 ) & 0x3F) as u8;
        let b5 = ( rgb & 0x1F) as u8;
        let r = (r5 << 3) | (r5 >> 2);
        let g = (g6 << 2) | (g6 >> 4);
        let b = (b5 << 3) | (b5 >> 2);
        Color { r, g, b, a: 0xFF }
    }

    pub fn from_u32(rgba: u32) -> Self {
        let r = ((rgba >> 24) & 0xFF) as u8;
        let g = ((rgba >> 16) & 0xFF) as u8;
        let b = ((rgba >>  8) & 0xFF) as u8;
        let a = ( rgba & 0xFF) as u8;

        Color { r, g, b, a }
    }

    pub fn from_u24(rgb24: u32) -> Self {
        let r = ((rgb24 >> 16) & 0xFF) as u8;
        let g = ((rgb24 >>  8) & 0xFF) as u8;
        let b = ( rgb24         & 0xFF) as u8;
        Color { r, g, b, a: 0xFF }
    }


}

#[derive(Debug, Copy, Clone)]
pub struct Size {
    pub absolute: Option<u32>,
    pub relative: Option<u32>,
}

impl Size {
    pub fn new(value: &str) -> Self {
        let mut absolute: Option<u32> = None;
        let mut relative: Option<u32> = None;

        if value.chars().last() == Some('%') {
            let value = &value[..value.len() - 1];
            relative = Some(value.parse::<u32>().expect("NIGGA WHUT"));
        } else {
            absolute = Some(value.parse::<u32>().expect("NIGGA WHUT"));
        }

        Size {
            absolute: absolute,
            relative: relative,
        }
    }

    pub fn from_u16(value: u16) -> Self {
        Size {
            absolute: Some(value as u32),
            relative: None,
        }
    }

    pub fn from_u32(value: u32) -> Self {
        Size {
            absolute: Some(value),
            relative: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Window {
    pub id: u16,
    pub name: String,
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub border_radius: u32,
    pub color: Color,
    pub children: Vec<Widget>,
    pub parent: ScreenStats,
    pub text_color: Color,
    pub display: Display,
    pub buffer: u32,
    pub action_bar: bool,
    pub wtype: Items,
}

#[derive(Clone, Debug)]
pub struct Button {
    pub id: u16,
    pub label: String,
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub color: Color,
    pub event: fn(&mut Widget, u32, u32, u32),
    pub padding: Size,
    pub border_radius: Size,
    pub text_color: Color,
    pub real_x: u32,
    pub real_y: u32,
    pub text_align: Align,
    pub args: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub id: u16,
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub color: Color,
    pub padding: Size,
    pub text_color: Color,
    pub border_radius: Size,
    pub children: Vec<Widget>,
    pub display: Display,
    pub real_x: u32,
    pub real_y: u32,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub id: u16,
    pub label: String,
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub color: Color,
    pub padding: Size,
    pub text_color: Color,
    pub border_radius: Size,
    pub real_x: u32,
    pub real_y: u32,
    pub ch_max: u32,
    pub ch_min: u32,
    pub text_align: Align,
}

#[derive(Clone, Debug)]
pub struct Image {
    pub id: u16,
    pub x: Size,
    pub y: Size,
    pub width: Size,
    pub height: Size,
    pub header: crate::targa::TargaHdr,
    pub padding: Size,
    pub ptr: u32,
    pub real_x: u32,
    pub real_y: u32,
    pub event: fn(&mut Widget, u32, u32, u32),
    pub args: [u32; 3],
}

#[derive(Clone, Debug)]
pub enum Widget {
    Window(Window),
    Frame(Frame),
    Button(Button),
    Label(Label),
    InputLabel(Label),
    ScrollFrame(Frame),
    Image(Image),
    Null(Null),
}

#[derive(Debug, Copy, Clone)]
pub struct Null {
    pub threads: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Display {
    Flex,
    Grid(Grid),
    None,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Grid {
    pub rows: usize,
    pub columns: usize,
}

impl Grid {
    pub fn new(columns: usize, rows: usize) -> Grid {
        Grid {
            columns: columns,
            rows: rows,
        }
    }
}

impl Window {
    pub fn new() -> Window {
        unsafe {
            (*(&raw mut SCREEN)).init();
            let width = (*(&raw mut SCREEN)).width;
            let height = (*(&raw mut SCREEN)).height;

            Window {
                id: libk::rng::LcgRng::global_new().range(0, 65545) as u16,
                name: String::from(""),
                x: Size::new("0"),
                y: Size::new("0"),
                width: Size::new("0"),
                height: Size::new("0"),
                border_radius: 0,
                children: Vec::new(),
                color: Color::rgb(255, 120, 56),
                text_color: Color::rgb(0, 0, 0),
                display: Display::None,
                buffer: 0,
                parent: ScreenStats {
                    depth: 0,
                    real_x: 0,
                    real_y: 0,
                    height: height,
                    width: width,
                },
                action_bar: true,
                wtype: Items::Window,
            }
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = String::from(name);

        self
    }

    pub fn w_type(mut self, t: Items) -> Self {
        self.wtype = t;

        self
    }

    pub fn action_bar(mut self, bool: bool) -> Self {
        self.action_bar = bool;

        self
    }

    pub fn x(mut self, x: Size) -> Self {
        self.x = x;

        match self.x.absolute {
            None => {
                self.x.absolute =
                    Some(self.parent.width.absolute.unwrap() / 100 * self.x.relative.unwrap());
            }
            _ => {}
        }

        self
    }

    pub fn y(mut self, y: Size) -> Self {
        self.y = y;

        match self.y.absolute {
            None => {
                self.y.absolute =
                    Some(self.parent.height.absolute.unwrap() / 100 * self.y.relative.unwrap());
            }
            _ => {}
        }

        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;

        match self.width.absolute {
            None => {
                self.width.absolute = Some(crate::kui_ceil(
                    self.parent.width.absolute.unwrap() as f32 / 100.0
                        * self.width.relative.unwrap() as f32,
                ) as u32);
            }
            _ => {}
        }

        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;

        match self.height.absolute {
            None => {
                self.height.absolute = Some(crate::kui_ceil(
                    self.parent.height.absolute.unwrap() as f32 / 100.0
                        * self.height.relative.unwrap() as f32,
                ) as u32);
            }
            _ => {
                if self.action_bar == true {
                    self.height.absolute = Some(self.height.absolute.unwrap())
                }
            }
        }

        self
    }

    pub fn display(mut self, display: Display) -> Self {
        self.display = display;

        self
    }

    pub fn get_width(&self) -> u32 {
        return self.width.absolute.unwrap();
    }

    pub fn get_height(&self) -> u32 {
        return self.height.absolute.unwrap();
    }

    pub fn border_radius(mut self, radius: u32) -> Self {
        self.border_radius = radius;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;

        self
    }

    pub fn get_x(&self) -> u32 {
        0
    }

    pub fn get_y(&self) -> u32 {
        0
    }

    pub fn add(&mut self, mut child: Widget) {
        child.reload(
            0,
            0,
            self.width.absolute.unwrap(),
            self.height.absolute.unwrap(),
            Display::None,
        );
        self.children.push(child);
    }

    pub fn add_exit(&mut self, child: Widget) {
        self.children.push(child);
    }

    pub fn to_window(&self) -> syscall::Window {
        syscall::Window {
            wid: self.id,
            x: self.x.absolute.unwrap() as u16,
            y: self.y.absolute.unwrap() as u16,
            z: 0,
            width: self.width.absolute.unwrap() as u16,
            height: self.height.absolute.unwrap() as u16,
            draw: crate::draw::draw_handler as u32,
            mouse: crate::draw::mouse_handler as u32,
            keyboard: unsafe { core::ptr::addr_of!(crate::draw::CHAR_BUFFER) as u32 },
            resize: crate::draw::resize_handler as u32,
            movable: self.action_bar,
            buffer: 0,
            wtype: self.wtype,
        }
    }
}

impl Frame {
    pub fn new() -> Frame {
        Frame {
            id: libk::rng::LcgRng::global_new().range(10, 65545) as u16,
            x: Size::new("0"),
            y: Size::new("0"),
            width: Size::new("0"),
            height: Size::new("0"),
            color: Color::rgb(255, 255, 255),
            border_radius: Size::new("0"),
            padding: Size::new("0"),
            text_color: Color::rgb(0, 0, 0),
            children: Vec::new(),
            display: Display::None,
            real_x: 0,
            real_y: 0,
        }
    }

    pub fn x(mut self, x: Size) -> Self {
        self.x = x;

        self
    }

    pub fn y(mut self, y: Size) -> Self {
        self.y = y;

        self
    }

    pub fn add(&mut self, child: Widget) {
        self.children.push(child);
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;

        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;

        self
    }

    pub fn border_radius(mut self, radius: Size) -> Self {
        self.border_radius = radius;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;

        self
    }

    pub fn padding(mut self, padding: Size) -> Self {
        self.padding = padding;

        self
    }

    pub fn display(mut self, display: Display) -> Self {
        self.display = display;

        self
    }

    pub fn get_padding(&self) -> u32 {
        return self.padding.absolute.unwrap();
    }

    pub fn get_width(&self) -> u32 {
        return self.width.absolute.unwrap();
    }

    pub fn get_height(&self) -> u32 {
        return self.height.absolute.unwrap();
    }

    pub fn get_x(&self) -> u32 {
        self.x.absolute.unwrap()
    }

    pub fn get_y(&self) -> u32 {
        self.y.absolute.unwrap()
    }

    pub fn reload(&mut self, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
        if display == Display::None || display == Display::Flex {
            if self.x.relative.is_some() {
                self.real_x = crate::kui_ceil(
                    px as f32 + pw as f32 / 100.0 * self.x.relative.unwrap() as f32,
                ) as u32;
            } else {
                self.real_x = px + self.x.absolute.unwrap();
            }

            if self.y.relative.is_some() {
                self.real_y = crate::kui_ceil(
                    py as f32 + ph as f32 / 100.0 * self.y.relative.unwrap() as f32,
                ) as u32;
            } else {
                self.real_y = py + self.y.absolute.unwrap();
            }

            match self.width.relative {
                None => {}
                _ => {
                    self.width.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.width.relative.unwrap() as f32,
                    ) as u32);
                }
            }

            match self.height.relative {
                None => {}
                _ => {
                    self.height.absolute = Some(crate::kui_ceil(
                        ph as f32 / 100.0 * self.height.relative.unwrap() as f32,
                    ) as u32);
                }
            }
        }

        match self.padding.relative {
            None => {}
            _ => {
                self.padding.absolute = Some(crate::kui_ceil(
                    pw as f32 / 100.0 * self.padding.relative.unwrap() as f32,
                ) as u32);
            }
        }

        match self.display {
            Display::Grid(g) => {
                for _ in
                    0..crate::kui_ceil(self.children.len() as f32 - (g.columns * g.rows) as f32)
                        as usize
                {
                    self.children.pop();
                }
            }

            _ => {}
        }
    }
}

impl Button {
    pub fn new() -> Button {
        Button {
            id: libk::rng::LcgRng::global_new().range(10, 65545) as u16,
            label: String::from(""),
            x: Size::new("0"),
            y: Size::new("0"),
            width: Size::new("0"),
            height: Size::new("0"),
            color: Color::rgb(255, 255, 255),
            event: do_nothing,
            padding: Size::new("0"),
            border_radius: Size::new("0"),
            text_color: Color::rgb(0, 0, 0),
            real_x: 0,
            real_y: 0,
            text_align: Align::Center,
            args: [0; 3],
        }
    }

    pub fn text_align(mut self, a: Align) -> Self {
        self.text_align = a;

        self
    }

    pub fn set_args(mut self, args: [u32; 3]) -> Self {
        self.args = args;

        self
    }

    pub fn x(mut self, x: Size) -> Self {
        self.x = x;

        self
    }

    pub fn y(mut self, y: Size) -> Self {
        self.y = y;

        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;

        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;

        self
    }

    pub fn border_radius(mut self, radius: Size) -> Self {
        self.border_radius = radius;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn event(mut self, event: fn(&mut Widget, u32, u32, u32)) -> Self {
        self.event = event;

        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;

        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = String::from(label);

        self
    }

    pub fn padding(mut self, padding: Size) -> Self {
        self.padding = padding;

        self
    }

    pub fn get_padding(&self) -> u32 {
        return self.padding.absolute.unwrap();
    }

    pub fn get_width(&self) -> u32 {
        return self.width.absolute.unwrap();
    }

    pub fn get_height(&self) -> u32 {
        return self.height.absolute.unwrap();
    }

    pub fn get_x(&self) -> u32 {
        self.x.absolute.unwrap()
    }

    pub fn get_y(&self) -> u32 {
        self.y.absolute.unwrap()
    }

    pub fn reload(&mut self, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
        if self.label == "x"
            && self.width.absolute.unwrap() == 19
            && self.height.absolute.unwrap() == 19
        {
            self.real_x = px + pw - 22;
            self.real_y = py - 25 + 3;
        } else {
            if display == Display::None {
                if self.x.relative.is_some() {
                    self.x.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.x.relative.unwrap() as f32,
                    ) as u32);
                }

                if self.y.relative.is_some() {
                    self.y.absolute = Some(crate::kui_ceil(
                        ph as f32 / 100.0 * self.y.relative.unwrap() as f32,
                    ) as u32);
                }

                self.real_x = px + self.x.absolute.unwrap();
                self.real_y = py + self.y.absolute.unwrap();

                match self.width.relative {
                    None => {}
                    _ => {
                        self.width.absolute = Some(crate::kui_ceil(
                            pw as f32 / 100.0 * self.width.relative.unwrap() as f32,
                        ) as u32);
                    }
                }

                match self.height.relative {
                    None => {}
                    _ => {
                        self.height.absolute = Some(crate::kui_ceil(
                            ph as f32 / 100.0 * self.height.relative.unwrap() as f32,
                        ) as u32);
                    }
                }
            }

            match self.padding.relative {
                None => {}
                _ => {
                    self.padding.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.padding.relative.unwrap() as f32,
                    ) as u32);
                }
            }
        }
    }
}

impl Label {
    pub fn new() -> Label {
        Label {
            id: libk::rng::LcgRng::global_new().range(10, 65535) as u16,
            label: String::from(""),
            x: Size::new("0"),
            y: Size::new("0"),
            width: Size::new("0"),
            height: Size::new("0"),
            color: Color::rgb(255, 255, 255),
            padding: Size::new("0"),
            border_radius: Size::new("0"),
            text_color: Color::rgb(0, 0, 0),
            real_x: 0,
            real_y: 0,

            ch_min: 0,
            ch_max: 0xFFFFFFFF,
            text_align: Align::Left,
        }
    }

    pub fn text_align(mut self, a: Align) -> Self {
        self.text_align = a;

        self
    }

    pub fn max(mut self, s: u32) -> Self {
        self.ch_max = s;

        self
    }

    pub fn min(mut self, s: u32) -> Self {
        self.ch_min = s;

        self
    }

    pub fn x(mut self, x: Size) -> Self {
        self.x = x;

        self
    }

    pub fn y(mut self, y: Size) -> Self {
        self.y = y;

        self
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;

        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;

        self
    }

    pub fn border_radius(mut self, radius: Size) -> Self {
        self.border_radius = radius;

        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;

        self
    }

    pub fn text_color(mut self, color: Color) -> Self {
        self.text_color = color;

        self
    }

    pub fn padding(mut self, padding: Size) -> Self {
        self.padding = padding;

        self
    }

    pub fn text(mut self, str: &str) -> Self {
        self.label = String::from(str);

        self
    }

    pub fn get_padding(&self) -> u32 {
        return self.padding.absolute.unwrap();
    }

    pub fn get_width(&self) -> u32 {
        return self.width.absolute.unwrap();
    }

    pub fn get_height(&self) -> u32 {
        return self.height.absolute.unwrap();
    }

    pub fn get_x(&self) -> u32 {
        self.x.absolute.unwrap()
    }

    pub fn get_y(&self) -> u32 {
        self.y.absolute.unwrap()
    }

    pub fn reload(&mut self, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
        if display == Display::None {

            if self.x.relative.is_some() {
                self.x.absolute = Some(crate::kui_ceil(
                    pw as f32 / 100.0 * self.x.relative.unwrap() as f32,
                ) as u32);
            }

            if self.y.relative.is_some() {
                self.y.absolute = Some(crate::kui_ceil(
                    ph as f32 / 100.0 * self.y.relative.unwrap() as f32,
                ) as u32);
            }

            self.real_x = px + self.x.absolute.unwrap();
            self.real_y = py + self.y.absolute.unwrap();

            match self.width.relative {
                None => {}
                _ => {
                    self.width.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.width.relative.unwrap() as f32,
                    ) as u32);
                }
            }

            match self.height.relative {
                None => {}
                _ => {
                    self.height.absolute = Some(crate::kui_ceil(
                        ph as f32 / 100.0 * self.height.relative.unwrap() as f32,
                    ) as u32);
                }
            }
        }

        match self.padding.relative {
            None => {}
            _ => {
                self.padding.absolute = Some(crate::kui_ceil(
                    pw as f32 / 100.0 * self.padding.relative.unwrap() as f32,
                ) as u32);
            }
        }
    }
}

impl Image {
    pub fn new(fname: &str) -> Image {
        let file = libk::io::File::new(fname);

        let header = unsafe { *(file.ptr as *const crate::targa::TargaHdr) };

        Image {
            id: libk::rng::LcgRng::global_new().range(10, 65535) as u16,
            x: Size::new("0"),
            y: Size::new("0"),
            width: Size::new("100%"),
            height: Size::new("100%"),
            header: header,
            padding: Size::new("0"),
            ptr: file.ptr,
            real_x: 0,
            real_y: 0,
            event: do_nothing,
            args: [0; 3],
        }
    }

    pub fn event(mut self, addr: fn(&mut Widget, u32, u32, u32)) -> Self {
        self.event = addr;

        self
    }

    pub fn set_args(mut self, args: [u32; 3]) -> Self {
        self.args = args;

        self
    }

    pub fn x(mut self, x: Size) -> Self {
        self.x = x;

        self
    }

    pub fn y(mut self, y: Size) -> Self {
        self.y = y;

        self
    }

    pub fn padding(mut self, padding: Size) -> Self {
        self.padding = padding;

        self
    }

    pub fn get_padding(&self) -> u32 {
        return self.padding.absolute.unwrap();
    }

    pub fn width(mut self, width: Size) -> Self {
        self.width = width;

        self
    }

    pub fn height(mut self, height: Size) -> Self {
        self.height = height;

        self
    }

    pub fn reload(&mut self, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
        if display == Display::None {
            if self.x.relative.is_some() {
                self.real_x = crate::kui_ceil(
                    px as f32 + pw as f32 / 100.0 * self.x.relative.unwrap() as f32,
                ) as u32;
            } else {
                self.real_x = px + self.x.absolute.unwrap();
            }

            if self.y.relative.is_some() {
                self.real_y = crate::kui_ceil(
                    py as f32 + ph as f32 / 100.0 * self.y.relative.unwrap() as f32,
                ) as u32;
            } else {
                self.real_y = py + self.y.absolute.unwrap();
            }

            match self.width.relative {
                None => {}
                _ => {
                    self.width.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.width.relative.unwrap() as f32,
                    ) as u32);
                }
            }

            match self.height.relative {
                None => {}
                _ => {
                    self.height.absolute = Some(crate::kui_ceil(
                        ph as f32 / 100.0 * self.height.relative.unwrap() as f32,
                    ) as u32);
                }
            }

            match self.padding.relative {
                None => {}
                _ => {
                    self.padding.absolute = Some(crate::kui_ceil(
                        pw as f32 / 100.0 * self.padding.relative.unwrap() as f32,
                    ) as u32);
                }
            }
        }
    }
}

pub fn do_nothing(s: &mut Widget, _arg1: u32, _arg2: u32, _arg3: u32) {}

impl Widget {

    pub fn get_label(&self) -> Option<&str> {
        match self {
            Widget::Button(val) => Some(&val.label),
            Widget::Label(val) => Some(&val.label),
            Widget::InputLabel(val) => Some(&val.label),
            _ => None,
        }
    }

    pub fn x(&self) -> u32 {
        match self {
            Widget::Button(val) => val.real_x,
            Widget::Label(val) => val.real_x,
            Widget::InputLabel(val) => val.real_x,
            Widget::Frame(val) => val.real_x,
            Widget::Image(val) => val.real_x,
            Widget::Window(val) => 0,
            _ => 0,
        }
    }

    pub fn y(&self) -> u32 {
        match self {
            Widget::Button(val) => val.real_y,
            Widget::Label(val) => val.real_y,
            Widget::InputLabel(val) => val.real_y,
            Widget::Frame(val) => val.real_y,
            Widget::Image(val) => val.real_y,
            Widget::Window(val) => 0,
            _ => 0,
        }
    }

    pub fn width(&self) -> u32 {
        match self {
            Widget::Button(val) => val.get_width(),
            Widget::Label(val) => val.get_width(),
            Widget::InputLabel(val) => val.get_width(),
            Widget::Frame(val) => val.get_width(),
            Widget::Window(val) => val.get_width(),
            Widget::Image(val) => val.width.absolute.unwrap(),
            _ => 0,
        }
    }

    pub fn height(&self) -> u32 {
        match self {
            Widget::Button(val) => val.get_height(),
            Widget::Label(val) => val.get_height(),
            Widget::InputLabel(val) => val.get_height(),
            Widget::Frame(val) => val.get_height(),
            Widget::Window(val) => val.get_height(),
            Widget::Image(val) => val.height.absolute.unwrap(),
            _ => 0,
        }
    }

    pub fn padding(&self) -> u32 {
        match self {
            Widget::Button(val) => val.get_padding(),
            Widget::Label(val) => val.get_padding(),
            Widget::InputLabel(val) => val.get_padding(),
            Widget::Frame(val) => val.get_padding(),
            Widget::Image(val) => val.get_padding(),
            Widget::Window(_) => 0,
            _ => 0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Widget::Button(val) => val.color,
            Widget::Label(val) => val.color,
            Widget::InputLabel(val) => val.color,
            Widget::Frame(val) => val.color,
            Widget::Window(val) => val.color,

            _ => Color::rgb(0, 0, 0),
        }
    }

    pub fn add(&mut self, child: Widget) {
        match self {
            Widget::Window(val) => val.add(child),
            Widget::Frame(val) => val.add(child),

            _ => {}
        }
    }

    pub fn reload(&mut self, px: u32, py: u32, pw: u32, ph: u32, display: Display) {
        match self {
            Widget::Frame(val) => val.reload(px, py, pw, ph, display),
            Widget::Button(val) => val.reload(px, py, pw, ph, display),
            Widget::Label(val) => val.reload(px, py, pw, ph, display),
            Widget::InputLabel(val) => val.reload(px, py, pw, ph, display),
            Widget::Image(val) => val.reload(px, py, pw, ph, display),

            _ => {}
        }
    }

    pub fn set_x(&mut self, sx: u32) {
        match self {
            Widget::Frame(val) => val.real_x = sx,
            Widget::Button(val) => val.real_x = sx,
            Widget::Label(val) => val.real_x = sx,
            Widget::InputLabel(val) => val.real_x = sx,
            Widget::Image(val) => val.real_x = sx,
            _ => {}
        }
    }

    pub fn set_y(&mut self, sy: u32) {
        match self {
            Widget::Frame(val) => val.real_y = sy,
            Widget::Button(val) => val.real_y = sy,
            Widget::Label(val) => val.real_y = sy,
            Widget::InputLabel(val) => val.real_y = sy,
            Widget::Image(val) => val.real_y = sy,
            _ => {}
        }
    }

    pub fn set_width(&mut self, sw: Size) {
        match self {
            Widget::Frame(val) => val.width = sw,
            Widget::Button(val) => val.width = sw,
            Widget::Label(val) => val.width = sw,
            Widget::InputLabel(val) => val.width = sw,
            Widget::Window(val) => val.width = sw,
            Widget::Image(val) => val.width = sw,
            _ => {}
        }
    }

    pub fn set_height(&mut self, sh: Size) {
        match self {
            Widget::Frame(val) => val.height = sh,
            Widget::Button(val) => val.height = sh,
            Widget::Label(val) => val.height = sh,
            Widget::InputLabel(val) => val.height = sh,
            Widget::Window(val) => val.height = sh,
            Widget::Image(val) => val.height = sh,
            _ => {}
        }
    }

    pub fn get_width(&self) -> Size {
        match self {
            Widget::Frame(val) => val.width,
            Widget::Button(val) => val.width,
            Widget::Label(val) => val.width,
            Widget::InputLabel(val) => val.width,
            Widget::Window(val) => val.width,
            Widget::Image(val) => val.width,

            _ => Size::new("0"),
        }
    }

    pub fn get_height(&self) -> Size {
        match self {
            Widget::Frame(val) => val.height,
            Widget::Button(val) => val.height,
            Widget::Label(val) => val.height,
            Widget::InputLabel(val) => val.height,
            Widget::Window(val) => val.height,
            Widget::Image(val) => val.height,

            _ => Size::new("0"),
        }
    }

    pub fn get_event(&self) -> fn(&mut Widget, u32, u32, u32) {
        match self {
            Widget::Button(val) => val.event.clone(),
            Widget::Image(val) => val.event.clone(),
            _ => do_nothing,
        }
    }

    pub fn get_x(&self) -> Size {
        match self {
            Widget::Frame(val) => val.x,
            Widget::Button(val) => val.x,
            Widget::Label(val) => val.x,
            Widget::InputLabel(val) => val.x,
            Widget::Image(val) => val.x,
            Widget::Window(_val) => Size::new("0"),

            _ => Size::new("0"),
        }
    }

    pub fn get_y(&self) -> Size {
        match self {
            Widget::Frame(val) => val.y,
            Widget::Button(val) => val.y,
            Widget::Label(val) => val.y,
            Widget::InputLabel(val) => val.y,
            Widget::Image(val) => val.y,
            Widget::Window(_val) => Size::new("0"),

            _ => Size::new("0"),
        }
    }

    pub fn get_id(&self) -> Option<u16> {
        match self {
            Widget::Frame(val) => Some(val.id),
            Widget::Button(val) => Some(val.id),
            Widget::Label(val) => Some(val.id),
            Widget::InputLabel(val) => Some(val.id),
            Widget::Image(val) => Some(val.id),
            Widget::Window(val) => Some(val.id),

            _ => None,
        }
    }
}
