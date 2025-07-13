use crate::display::{Color, DisplayServer, Mouse, State};
use core::sync::atomic::{AtomicU16, Ordering};

pub static mut DISPLAY_SERVER: DisplayServer = DisplayServer {
    width: 0,
    height: 0,
    pitch: 0,
    depth: 8,

    framebuffer: 0xFD000000,
    double_buffer: 0x00,
};

pub static mut MOUSE: Mouse = Mouse {
    x: 0,
    y: 0,

    left: false,
    center: false,
    right: false,

    state: State::Point,
};

const O: u32 = 0x0000_0000;
const B: u32 = 0x0000_00FF;
const T: u32 = 0xFFFF_FFFF;

const MOUSE_CURSOR: [u32; 96] = [
    B, O, O, O, O, O, O, O,
    B, B, O, O, O, O, O, O,
    B, T, B, O, O, O, O, O,
    B, T, T, B, O, O, O, O,
    B, T, T, T, B, O, O, O,
    B, T, T, T, T, B, O, O,
    B, T, T, T, T, T, B, O,
    B, T, T, T, T, T, T, B,
    B, T, T, T, B, B, B, B,
    B, T, B, B, T, B, O, O,
    B, B, O, O, B, T, B, O,
    B, O, O, O, O, B, B, O,
];

impl DisplayServer {
    pub fn init(&mut self) {
        let vbe = unsafe { crate::BOOTINFO.mode };
        self.width = vbe.width as u64;
        self.pitch = vbe.pitch as u64;
        self.height = vbe.height as u64;
        self.depth = vbe.bpp as usize;

        unsafe {
            crate::display::DEPTH = vbe.bpp;
        }

        self.framebuffer = vbe.framebuffer;
        unsafe {
            (*(&raw mut crate::pmm::PADDR))
                .add_fb(self.framebuffer, self.pitch as u32 * self.height as u32);

            self.double_buffer = (*(&raw mut crate::pmm::PADDR))
                .malloc(self.pitch as u32 * self.height as u32)
                .unwrap();
        }
    }

    pub fn copy(&self) {
        let buffer_size = self.pitch as u32 * self.height as u32;
        unsafe {
            core::ptr::copy(
                self.double_buffer as *const u8,
                self.framebuffer as *mut u8,
                buffer_size as usize,
            );
        }
    }

    pub fn copy_to_fb(&self, x: u32, y: u32, width: u32, height: u32) {
        let bytes_per_pixel = match self.depth {
            32 => 4,
            24 => 3,
            _  => return,
        };

        let src = self.double_buffer as *const u8;
        let dst = self.framebuffer   as *mut u8;
        let pitch = self.pitch as u32;

        unsafe {
            for row in 0..height {
                let line_start = (y + row) * pitch;
                let offset = line_start + x * bytes_per_pixel;

                core::ptr::copy(
                    src.add(offset as usize),
                    dst.add(offset as usize),
                    (width * bytes_per_pixel) as usize,
                );
            }
        }
    }

    pub fn copy_to_db(&self, width: u32, height: u32, buffer: u32, x: u32, y: u32) {
        let bpp = self.depth;
        if bpp != 32 && bpp != 24 {
            return;
        }

        let bytes_per_pixel = if bpp == 32 { 4 } else { 3 };
        let dst_pitch = (self.pitch as usize);  // bytes per scanline
        let src_pitch = (width as usize) * bytes_per_pixel;

        for row in 0..height as usize {
            for col in 0..width as usize {
                let dst_x = x as usize + col;
                let dst_y = y as usize + row;

                if dst_x < self.width as usize && dst_y < self.height as usize {
                    let dst_index = dst_y * dst_pitch + dst_x * bytes_per_pixel;
                    let src_index = row * src_pitch + col * bytes_per_pixel;

                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            (buffer as *const u8).add(src_index),
                            (self.double_buffer as *mut u8).add(dst_index),
                            bytes_per_pixel,
                        );
                    }
                }
            }
        }
    }

    pub fn copy_to_fb_a(&self, width: u32, height: u32, buffer: u32, x: u32, y: u32) {
        let bpp = self.depth;
        if bpp != 32 && bpp != 24 {
            return;
        }

        let bytes_per_pixel = if bpp == 32 { 4 } else { 3 };
        let dst_pitch = self.pitch as usize;

        for row in 0..height as usize {
            for col in 0..width as usize {
                let dst_x = x as usize + col;
                let dst_y = y as usize + row;

                if dst_x < self.width as usize && dst_y < self.height as usize {
                    let dst_index = dst_y * dst_pitch + dst_x * bytes_per_pixel;
                    let src_index = row * (width as usize * bytes_per_pixel) + col * bytes_per_pixel;

                    unsafe {
                        let mut src_pixel = [0u8; 4];
                        core::ptr::copy_nonoverlapping(
                            (buffer as *const u8).add(src_index),
                            src_pixel.as_mut_ptr(),
                            bytes_per_pixel,
                        );

                        if bpp == 32 {
                            let alpha = src_pixel[3] as u16;
                            let inv = 255 - alpha;

                            let mut dst_pixel = [0u8; 4];
                            core::ptr::copy_nonoverlapping(
                                (self.framebuffer as *const u8).add(dst_index),
                                dst_pixel.as_mut_ptr(),
                                4,
                            );
                            // blend R,G,B channels
                            for i in 0..3 {
                                let blended = (src_pixel[i] as u16 * alpha
                                    + dst_pixel[i] as u16 * inv) / 255;
                                dst_pixel[i] = blended as u8;
                            }
                            // write back
                            core::ptr::copy_nonoverlapping(
                                dst_pixel.as_ptr(),
                                (self.framebuffer as *mut u8).add(dst_index),
                                4,
                            );
                        } else {
                            // 24bpp: no alpha to blend, just overwrite
                            core::ptr::copy_nonoverlapping(
                                src_pixel.as_ptr(),
                                (self.framebuffer as *mut u8).add(dst_index),
                                3,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn write_pixel(&self, row: u32, col: u32, color: Color) {
        if col < self.width as u32 && row < self.height as u32 {
            unsafe {
                match self.depth {
                    16 => {
                        *((self.framebuffer as *mut u16).add((row * self.width as u32 + col) as usize)) = color.to_u16();
                    },

                    24 => {
                        let color = color.to_u24();
                        *((self.framebuffer as *mut u8).add(((row * self.width as u32 + col) * 3 + 0) as usize)) = color[0];
                        *((self.framebuffer as *mut u8).add(((row * self.width as u32 + col) * 3 + 1) as usize)) = color[1];
                        *((self.framebuffer as *mut u8).add(((row * self.width as u32 + col) * 3 + 2) as usize)) = color[2];
                    }

                    32 => {
                        *((self.framebuffer as *mut u32).add((row * self.width as u32 + col) as usize)) = color.to_u32();
                    }

                    _ => {}
                }
            }
        }
    }

    pub fn draw_mouse(&self, x: u16, y: u16) {

        for i in 0..12 {
            for j in 0..8 {
                let color = MOUSE_CURSOR[(i * 8 + j) as usize];
                if color != O {
                    self.write_pixel(
                        y.wrapping_add(i) as u32,
                        x.wrapping_add(j) as u32,
                        Color::from_u32(color),
                    );
                }
            }
        }
    }
}

pub static mut LAST_INPUT: u8 = 0;
pub static mut DRAGS: u8 = 0;
pub static mut DRAG: bool = false;
pub static mut DRAGGING_WINDOW: AtomicU16 = AtomicU16::new(0);
pub static mut RESIZING_WINDOW: AtomicU16 = AtomicU16::new(0);

pub static mut W_WIDTH: u16 = 0;
pub static mut W_HEIGHT: u16 = 0;

impl Mouse {
    pub fn cursor(&mut self, data: [u8; 3]) {
        unsafe { (*(&raw mut DISPLAY_SERVER)).copy_to_fb(self.x as u32, self.y as u32, 8, 12) };

        let x_vec = (data[1] as i8) as i16;
        let y_vec = (data[2] as i8) as i16;

        self.x = self.clamp_mx(x_vec);
        self.y = self.clamp_my(-y_vec);

        self.left = (data[0] & 0b00000001) != 0;
        self.right = (data[0] & 0b00000010) != 0;
        self.center = (data[0] & 0b00000100) != 0;

        unsafe {
            LAST_INPUT = data[0];
        }
        unsafe { (*(&raw mut DISPLAY_SERVER)).draw_mouse(self.x, self.y) };

        unsafe {
            if (LAST_INPUT & 0b00000001) != 0 {
                DRAGS = DRAGS.wrapping_add(1);

                if DRAGS > 1 {
                    DRAG = true;
                }
            } else {
                DRAGS = 0;
                DRAG = false;

                if (*(&raw mut RESIZING_WINDOW)).load(Ordering::Relaxed) != 0 {
                    let w = (*(&raw mut COMPOSER))
                        .find_window_id((*(&raw mut RESIZING_WINDOW)).load(Ordering::Relaxed))
                        .unwrap();
                    (*(&raw mut crate::pmm::PADDR)).dealloc(w.buffer);

                    w.width = W_WIDTH;
                    w.height = W_HEIGHT;

                    let tot_size =
                        w.width as u32 * w.height as u32 * (DISPLAY_SERVER.depth / 4) as u32;
                    w.buffer = (*(&raw mut crate::pmm::PADDR)).malloc(tot_size).unwrap();

                    (*(&raw mut DRAGGING_WINDOW)).store(0, Ordering::Relaxed);
                    (*(&raw mut RESIZING_WINDOW)).store(0, Ordering::Relaxed);

                    W_WIDTH = 0;
                    W_HEIGHT = 0;

                    (*(&raw mut crate::task::TASK_MANAGER))
                        .lock()
                        .add_user_task(
                            w.resize,
                            Some(&[w.wid as u32, w.width as u32, w.height as u32, w.buffer]),
                        );

                } else if (*(&raw mut DRAGGING_WINDOW)).load(Ordering::Relaxed) != 0 {
                    (*(&raw mut DRAGGING_WINDOW)).store(0, Ordering::Relaxed);
                    (*(&raw mut RESIZING_WINDOW)).store(0, Ordering::Relaxed);
                    W_WIDTH = 0;
                    W_HEIGHT = 0;

                    for i in (0..(*(&raw mut COMPOSER)).windows.len()).rev() {
                        let ty = COMPOSER.windows[i].wtype;
                        if ty != Items::Null {
                            (*(&raw mut DISPLAY_SERVER)).copy_to_db(
                                COMPOSER.windows[i].width as u32,
                                COMPOSER.windows[i].height as u32,
                                COMPOSER.windows[i].buffer,
                                COMPOSER.windows[i].x as u32,
                                COMPOSER.windows[i].y as u32,
                            );
                        }
                    }
                    (*(&raw mut DISPLAY_SERVER)).copy();
                }

                return;
            }
        }

        if self.left {
            let w = unsafe { (*(&raw mut COMPOSER)).find_window(self.x, self.y) };

            if unsafe { (*(&raw mut RESIZING_WINDOW)).load(Ordering::Relaxed) != 0 } {
                let dx = x_vec;
                let dy = y_vec * -1;

                let w = unsafe {
                    (*(&raw mut COMPOSER))
                        .find_window_id((*(&raw mut RESIZING_WINDOW)).load(Ordering::Relaxed))
                        .unwrap()
                };

                let final_width = self.rem_sign(unsafe { W_WIDTH } as i16 + dx);
                let final_height = self.rem_sign(unsafe { W_HEIGHT } as i16 + dy);

                unsafe {
                    if W_WIDTH <= final_width && W_HEIGHT <= final_height {
                        (*(&raw mut DISPLAY_SERVER)).copy_to_fb(
                            w.x as u32,
                            w.y as u32,
                            final_width as u32,
                            final_height as u32,
                        );
                    } else {
                        let mut ww = final_width;
                        let mut wh = final_height;

                        if W_WIDTH > final_width {
                            ww = W_WIDTH + 1;
                        }

                        if W_HEIGHT > final_height {
                            wh = W_HEIGHT + 1;
                        }

                        (*(&raw mut DISPLAY_SERVER))
                            .copy_to_fb(w.x as u32, w.y as u32, ww as u32, wh as u32);
                    }
                }

                unsafe {
                    W_WIDTH = cap(
                        final_width as usize,
                        ((*(&raw mut DISPLAY_SERVER)).width - w.x as u64) as usize,
                    ) as u16;
                    W_HEIGHT = cap(
                        final_height as usize,
                        ((*(&raw mut DISPLAY_SERVER)).height - w.y as u64) as usize,
                    ) as u16;
                }

                self.draw_square_outline(
                    w.y,
                    w.x,
                    unsafe { W_HEIGHT },
                    unsafe { W_WIDTH },
                    Color::rgb(245, 245, 247),
                );

                return;

            } else if unsafe { (*(&raw mut DRAGGING_WINDOW)).load(Ordering::Relaxed) != 0 } {
                let composer = unsafe { &raw mut COMPOSER };
                let display_server = unsafe { &raw mut DISPLAY_SERVER };

                let window_opt = unsafe {
                    (*composer)
                        .find_window_id((*(&raw mut DRAGGING_WINDOW)).load(Ordering::Relaxed))
                };
                let w = match window_opt {
                    Some(w) => w,
                    None => return,
                };

                let old_x = w.x;
                let old_y = w.y;

                let new_x = add_delta(old_x, x_vec);
                let new_y = add_delta(old_y, -y_vec);

                let mut updated_x = old_x;
                let mut updated_y = old_y;

                if (new_x as i16 + w.width as i16)
                    <= unsafe { ((*display_server).width - 1) as i16 }
                {
                    updated_x = new_x;
                }

                if (new_y as i16 + w.height as i16)
                    <= unsafe { ((*display_server).height + 24) as i16 }
                {
                    updated_y = new_y;
                }

                let reset_rect = self.union_rect(
                    old_x as u32,
                    old_y as u32,
                    w.width as u32,
                    w.height as u32,
                    updated_x as u32,
                    updated_y as u32,
                );

                unsafe {
                    (*display_server).copy_to_fb(
                        reset_rect.0,
                        reset_rect.1,
                        reset_rect.2,
                        reset_rect.3,
                    );
                }

                w.x = updated_x;
                w.y = updated_y;

                unsafe {
                    (*composer)
                        .copy_window_fb((*(&raw mut DRAGGING_WINDOW)).load(Ordering::Relaxed))
                };
                return;
            }

            if let Some(ws) = w {
                let w_type = ws.wtype;
                if w_type == Items::Window && ws.z != 0 && unsafe { DRAG == false } {
                    let x = ws.x;
                    let y = ws.y;
                    let width = ws.width;
                    let height = ws.height;
                    let id = ws.wid;

                    unsafe {
                        for i in (*(&raw mut COMPOSER)).windows.iter_mut() {
                            if i.wid != id {
                                i.z = i.z.wrapping_add(1);
                            } else {
                                i.z = 0;
                            }
                        }

                        (*(&raw mut COMPOSER)).windows.sort_by_key(|w| w.z);
                        (*(&raw mut COMPOSER)).copy_window(id);
                    }

                    unsafe {
                        (*(&raw mut DISPLAY_SERVER)).copy_to_fb(
                            x as u32,
                            y as u32,
                            width as u32,
                            height as u32,
                        )
                    };
                } else {
                    if ws.movable && self.y >= ws.y && self.y <= ws.y + 25 {
                        if unsafe { DRAG == true } {
                            unsafe {
                                (*(&raw mut DRAGGING_WINDOW)).store(0, Ordering::Relaxed);
                                (*(&raw mut RESIZING_WINDOW)).store(0, Ordering::Relaxed);
                                W_WIDTH = 0;
                                W_HEIGHT = 0;

                                (*(&raw mut DRAGGING_WINDOW)).store(ws.wid + 0, Ordering::Relaxed)
                            };
                            return;
                        } else if ws.mouse != 0 {
                            let xc = ws.x;
                            let yc = ws.y;
                            let id = ws.wid;
                            let mouse = ws.mouse;

                            unsafe {
                                (*(&raw mut crate::task::TASK_MANAGER))
                                    .lock()
                                    .add_user_task(
                                        mouse,
                                        Some(&[
                                            id as u32,
                                            (self.x - xc) as u32,
                                            (self.y - yc) as u32,
                                        ]),
                                    );
                            }
                        }
                    } else if ws.movable
                        && (self.is_bottom_right(ws.x, ws.y, ws.width, ws.height, self.x, self.y))
                    {
                        if unsafe { DRAG } == true {
                            if unsafe { (*(&raw mut RESIZING_WINDOW)).load(Ordering::Relaxed) == 0 }
                            {
                                unsafe {
                                    W_WIDTH = ws.width;
                                    W_HEIGHT = ws.height;
                                    (*(&raw mut RESIZING_WINDOW)).store(ws.wid, Ordering::Relaxed);
                                };
                            }
                        }
                    } else {
                        if ws.mouse != 0 && unsafe { DRAG == false } {
                            let xc = ws.x;
                            let yc = ws.y;
                            let id = ws.wid;
                            let mouse = ws.mouse;

                            unsafe {
                                (*(&raw mut crate::task::TASK_MANAGER))
                                    .lock()
                                    .add_user_task(
                                        mouse,
                                        Some(&[
                                            id as u32,
                                            (self.x - xc) as u32,
                                            (self.y - yc) as u32,
                                        ]),
                                    );
                            };
                        }
                    }
                }
            }
        }
    }

    fn rem_sign(&self, n: i16) -> u16 {
        if n < 0 { (n * -1) as u16 } else { n as u16 }
    }

    pub fn union_rect(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
        x2: u32,
        y2: u32,
    ) -> (u32, u32, u32, u32) {
        let min_x = x.min(x2);
        let max_x = (x + width).max(x2 + width);
        let min_y = y.min(y2);
        let max_y = (y + height).max(y2 + height);

        let width = max_x - min_x;
        let height = max_y - min_y;

        (min_x, min_y, width, height)
    }

    fn is_bottom_right(
        &self,
        w_x: u16,
        w_y: u16,
        w_width: u16,
        w_height: u16,
        mouse_x: u16,
        mouse_y: u16,
    ) -> bool {
        let x_min = w_x.wrapping_add(w_width.wrapping_sub(8));
        let x_max = w_x.wrapping_add(w_width.wrapping_sub(0));
        let y_min = w_y.wrapping_add(w_height.wrapping_sub(8));
        let y_max = w_y.wrapping_add(w_height.wrapping_sub(0));

        (mouse_x >= x_min && mouse_x <= x_max) && (mouse_y >= y_min && mouse_y <= y_max)
    }

    fn clamp_mx(&self, n: i16) -> u16 {
        let mx_0 = self.x as i16;
        let sx = unsafe { (*(&raw mut DISPLAY_SERVER)).width } as u16;

        if n + mx_0 >= (sx as i16 - 8) {
            sx.wrapping_sub(8)
        } else if n + mx_0 <= 0 {
            0
        } else {
            (n + mx_0) as u16
        }
    }

    pub fn draw_square_outline(&self, x: u16, y: u16, width: u16, height: u16, color: Color) {
        let max_x = x + width - 1;
        let max_y = y + height - 1;

        unsafe {
            for i in x..=max_x {
                (*(&raw mut DISPLAY_SERVER)).write_pixel(i as u32, y as u32, color);
                (*(&raw mut DISPLAY_SERVER)).write_pixel(i as u32, max_y as u32, color);
            }

            for i in y..=max_y {
                (*(&raw mut DISPLAY_SERVER)).write_pixel(x as u32, i as u32, color);
                (*(&raw mut DISPLAY_SERVER)).write_pixel(max_x as u32, i as u32, color);
            }
        }
    }

    pub fn clamp_my(&self, n: i16) -> u16 {
        let my_0 = self.y as i16;
        let sy = unsafe { (*(&raw mut DISPLAY_SERVER)).height } as u16;

        if n + my_0 >= (sy as i16 - 12) {
            sy.wrapping_sub(12)
        } else if n + my_0 <= 0 {
            return 0;
        } else {
            (n + my_0) as u16
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C)]
pub enum Items {
    Wallpaper,
    Bar,
    Popup,
    Window,
    Null,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Window {
    pub wid: u16,
    pub x: u16,
    pub y: u16,
    pub z: u16,
    pub width: u16,
    pub height: u16,
    draw: u32,
    mouse: u32,
    keyboard: u32,
    pub resize: u32,
    movable: bool,
    pub buffer: u32,
    pub wtype: Items,
}

pub static NULL_WINDOW: Window = Window {
    wid: 0,
    x: 0,
    y: 0,
    z: 255,
    width: 0,
    height: 0,
    draw: 0,
    mouse: 0,
    keyboard: 0,
    resize: 0,
    movable: false,
    buffer: 0,
    wtype: Items::Null,
};

#[derive(Debug, Clone)]
pub struct Composer {
    pub windows: [Window; 16],
}

pub static mut COMPOSER: Composer = Composer {
    windows: [NULL_WINDOW; 16],
};

impl Composer {
    pub fn copy_window(&mut self, id: u16) {
        for i in 0..self.windows.len() {
            if id == self.windows[i].wid {
                match self.windows[i].wtype {
                    Items::Null => {}
                    _ => unsafe {
                        (*(&raw mut crate::composer::DISPLAY_SERVER)).copy_to_db(
                            self.windows[i].width as u32,
                            self.windows[i].height as u32,
                            self.windows[i].buffer as u32,
                            self.windows[i].x as u32,
                            self.windows[i].y as u32,
                        )
                    },
                }
            }
        }
    }

    pub fn copy_window_fb(&mut self, id: u16) {
        for i in 0..self.windows.len() {
            if id == self.windows[i].wid {
                match self.windows[i].wtype {
                    Items::Null => {}
                    _ => unsafe {
                        (*(&raw mut DISPLAY_SERVER)).copy_to_fb_a(
                            self.windows[i].width as u32,
                            self.windows[i].height as u32,
                            self.windows[i].buffer as u32,
                            self.windows[i].x as u32,
                            self.windows[i].y as u32,
                        )
                    },
                }
            }
        }
    }

    pub fn find_window(&mut self, x: u16, y: u16) -> Option<&mut Window> {
        for i in 0..self.windows.len() {
            if x >= self.windows[i].x
                && x <= (self.windows[i].x + self.windows[i].width)
                && y >= self.windows[i].y
                && y <= (self.windows[i].y + self.windows[i].height)
            {
                match self.windows[i].wtype {
                    Items::Null => {}
                    _ => return Some(&mut self.windows[i]),
                }
            }
        }

        None
    }

    pub fn find_window_id(&mut self, id: u16) -> Option<&mut Window> {
        for i in 0..self.windows.len() {
            if self.windows[i].wid == id {
                let h = self.windows[i].wtype;
                if h != Items::Null {
                    return Some(&mut self.windows[i]);
                }
            }
        }

        None
    }

    pub fn check_id(&self, mut rng: libk::rng::LcgRng) -> u16 {
        loop {
            let wid = rng.range(0, 65545) as u16;

            let mut is_used = false;
            for i in 0..self.windows.len() {
                if self.windows[i].wid == wid {
                    is_used = true;
                    break;
                }
            }

            if !is_used {
                return wid;
            }
        }
    }

    pub fn add_window(&mut self, mut w: Window) -> (u32, u32) {
        let wtype = w.wtype;
        if wtype == Items::Wallpaper {
            w.z = 254;
        } else if wtype == Items::Bar {
            w.z = 0;
        } else if wtype == Items::Popup {
            w.z = 0;
        }

        w.buffer = unsafe {
            (*(&raw mut crate::pmm::PADDR))
                .malloc(
                    w.width as u32
                        * w.height as u32
                        * (((*(&raw mut DISPLAY_SERVER)).depth / 4) as u32),
                )
                .unwrap()
        };

        let rng = libk::rng::LcgRng::new(w.buffer as u64);
        w.wid = self.check_id(rng);

        for i in 0..self.windows.len() {
            match self.windows[i].wtype {
                Items::Null => {
                    self.windows[i] = w;
                    break;
                }
                _ => {}
            }
        }

        for i in 0..self.windows.len() {
            if self.windows[i].wid != w.wid {
                self.windows[i].z = self.windows[i].z.wrapping_add(1);
            }
        }

        self.windows.sort_by_key(|w| w.z);
        unsafe {
            (*(&raw mut DRAGGING_WINDOW)).store(0, Ordering::Relaxed);
            (*(&raw mut RESIZING_WINDOW)).store(0, Ordering::Relaxed);
            W_WIDTH = 0;
            W_HEIGHT = 0;
        }

        (w.wid as u32, w.buffer)
    }

    pub fn write_kb(&mut self, char: char) {
        for i in 0..self.windows.len() {
            let y = self.windows[i].wtype;

            if self.windows[i].z == 0
                && y != Items::Bar
                && y != Items::Wallpaper
                && y != Items::Null
                && self.windows[i].keyboard != 0
            {
                for j in 0..64 {
                    unsafe {
                        if *((j + self.windows[i].keyboard) as *const u8) == 0 {
                            *((j + self.windows[i].keyboard) as *mut u8) = char as u8;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn remove_window(&mut self, wid: u16) {
        for i in 0..self.windows.len() {
            if self.windows[i].wid == wid {
                unsafe { (*(&raw mut crate::pmm::PADDR)).dealloc(self.windows[i].buffer) };
                self.windows[i].wtype = Items::Null;
                self.windows[i].z = 255;
            }
        }

        self.windows.sort_by_key(|w| w.z);

        unsafe {
            for j in (0..self.windows.len()).rev() {
                match self.windows[j].wtype {
                    Items::Null => {}
                    _ => {
                        (*(&raw mut DISPLAY_SERVER)).copy_to_db(
                            self.windows[j].width as u32,
                            self.windows[j].height as u32,
                            self.windows[j].buffer,
                            self.windows[j].x as u32,
                            self.windows[j].y as u32,
                        );
                    }
                }
            }

            (*(&raw mut DISPLAY_SERVER)).copy();
        }
    }
}

fn cap(n: usize, value: usize) -> usize {
    if n > value { value } else { n }
}

pub fn add_delta(n: u16, m: i16) -> u16 {
    if (n as i16 + m) < 0 {
        0
    } else {
        (n as i16 + m) as u16
    }
}
