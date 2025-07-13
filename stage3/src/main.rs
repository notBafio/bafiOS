#![no_std]
#![no_main]

mod disk;

use core::arch::asm;
use core::fmt;

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub extern "C" fn _start() -> ! {
    let ebx: u16;

    unsafe {
        asm!(
            "mov {0:e}, 0x10",
            "mov ds, {0:e}",
            "mov es, {0:e}",
            "mov ss, {0:e}",

            "mov esp, {1:e}",

            out(reg) _,
            in(reg) 0x30_0000,
            out("ebx") ebx,

            options(nostack),
        );
    }

    let target = 0x10_0000 as *mut u8;
    disk::read(4096, 2048, target);

    println!("[+] Jumping to kernel ...");

    unsafe {
        asm!(
            "mov ebx, {1:e}",
            "call {0:e}",
            in(reg) 0x10_0000,
            in(reg) ebx,

        );
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    println!("[x] Bootloader panicked at stage 3! x_x");
    loop {}
}

/*
    Debug print macros from libk
*/
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
}

impl fmt::Write for Terminal {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::_print(format_args!($($arg)*)));
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

pub fn outb(port: u16, value: u8) {
    unsafe {
        asm!(
            "out dx, al",
            in("dx") port,
            in("al") value,
            options(nomem, nostack, preserves_flags));
    }
}
