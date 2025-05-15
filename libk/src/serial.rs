use core::fmt;

use crate::port::{inb, outb};

pub struct Terminal {}

impl Terminal {
    pub fn new() -> Self {
        Terminal {}
    }

    pub fn write_byte(&self, byte: u8) {
        match byte {
            b'\n' => outb(0x3F8, '\n' as u8),
            byte => {
                outb(0x3F8, byte);
            }
        }
    }

    pub fn write_string(&self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn write_kb(&self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => {
                    self.write_byte(byte);
                }

                _ => {
                    self.write_byte(0xfe);
                }
            }
        }
    }
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::serial::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    Terminal::new().write_fmt(args).unwrap();
}
