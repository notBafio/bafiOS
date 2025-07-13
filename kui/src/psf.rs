use alloc::vec::Vec;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct PSF1 {
    magic: u16,
    mode: u8,
    ch_size: u8,
}

pub struct Font {
    addr: u32,
    size: u32,
    ftype: u32,
    chars: u32,
    pub hdr1: PSF1,
}

#[derive(Debug, Clone)]
pub struct Glyph {
    pub height: u32,
    pub width: u32,
    pub map: Vec<bool>,
}

pub static mut FONT: Font = Font {
    addr: 0,
    size: 0,
    ftype: 0,
    chars: 0,
    hdr1: PSF1 {
        magic: 0,
        mode: 0,
        ch_size: 0,
    },
};

impl Font {
    pub fn load(&mut self) {
        let file = libk::io::File::new("SYS/FONT/DEFAULT.PSF");

        let hdr = unsafe { *(file.ptr as *const PSF1) };

        if hdr.magic == 0x0436 {
            self.addr = file.ptr;
            self.chars = if hdr.mode == 0x1 { 256 } else { 512 };
            self.size = hdr.ch_size as u32;
            self.ftype = 1;
            self.hdr1 = hdr;
        } else {
            libk::println!("Invalid font file");
            return;
        }
    }

    pub fn get_char(&mut self, char: char) -> Glyph {
        if self.addr == 0 || self.hdr1.magic != 0x0436 {
            self.load();
        }

        let mut glyph = Glyph {
            height: self.size,
            width: 8,
            map: Vec::new(),
        };

        let lines = unsafe {
            core::slice::from_raw_parts(
                (self.addr as usize + 4 + char as usize * self.size as usize) as *const u8,
                self.size as usize,
            )
        };

        for i in 0..self.size as usize {
            for j in (0..8).rev() {
                let bit = ((lines[i] >> j) & 1) != 0;
                glyph.map.push(bit);
            }
        }

        glyph
    }

    pub fn unload(&mut self) {
        libk::syscall::free(self.addr);

        self.addr = 0;
    }
}
