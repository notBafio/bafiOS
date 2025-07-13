#![no_std]
#![no_main]

mod disk;
mod gdt;
mod tss;

use core::fmt;
use core::ptr::addr_of;
use gdt::GDT;

use core::arch::asm;
use core::panic::PanicInfo;

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MemoryMapEntry {
    base: u64,
    length: u64,
    memory_type: u32,
    reserved_acpi: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct MemoryMap {
    entries: [MemoryMapEntry; 32],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct Rsdp {
    signature: [u8; 8],
    checksum: u8,
    oem_id: [u8; 6],
    revision: u8,
    rsdt_address: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct BootInfo {
    mmap: MemoryMap,
    rsdp: Rsdp,
    tss: u16,
    vbe: VbeInfoBlock,
    mode: VbeModeInfoBlock,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VbeInfoBlock {
    signature: [u8; 4],
    version: u16,
    oem: [u16; 2],
    dunno: [u8; 4],
    video_ptr: u32,
    memory_size: u16,
    reserved: [u8; 492],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct VbeModeInfoBlock {
    attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    pitch: u16,
    width: u16,
    height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    bpp: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask_size: u8,
    red_field_position: u8,
    green_mask_size: u8,
    green_field_position: u8,
    blue_mask_size: u8,
    blue_field_position: u8,
    reserved_mask_size: u8,
    reserved_field_position: u8,
    direct_color_mode_info: u8,
    framebuffer: u32,
    reserved1: u32,
    reserved2: u16,
    lin_bytes_per_scan_line: u16,
    bnk_image_pages: u8,
    lin_image_pages: u8,
    lin_red_mask_size: u8,
    lin_red_field_position: u8,
    lin_green_mask_size: u8,
    lin_green_field_position: u8,
    lin_blue_mask_size: u8,
    lin_blue_field_position: u8,
    lin_reserved_mask_size: u8,
    lin_reserved_field_position: u8,
    max_pixel_clock: u32,
    reserved3: [u8; 189],
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub extern "C" fn _start() -> ! {
    Terminal::new().write_string("[+] Loading stage 3 ...");

    disk::read_stub();

    Terminal::new().write_string("[+] Jumping to protected mode ...");

    protected_mode();

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    Terminal::new().write_string("[x] Bootloader stage 2 panicked! x_x");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn fail() -> ! {
    Terminal::new().write_string("[x] Disk reading failed! x_x");
    loop {}
}

static mut BOOT: BootInfo = unsafe { core::mem::zeroed() };
static mut VBE_MODE: VbeModeInfoBlock = unsafe { core::mem::zeroed() };

fn protected_mode() {
    unsafe {
        let tss_addr = (*(&raw mut GDT)).write_tss();
        (*(&raw mut GDT)).load();

        BOOT.rsdp = get_rsdp();
        BOOT.vbe = get_vbe_info();
        BOOT.tss = tss_addr;
        get_mmap();

        let best_mode = find_vbe_mode();

        asm!(
            "int 0x10",

            in("ax") 0x4F02,
            in("bx") best_mode
        );

        asm!("mov eax, cr0", "or eax, 1 << 0", "mov cr0, eax",);

        asm!("mov bx, {0:x}", in(reg) addr_of!(BOOT) as u16);

        asm!("ljmp $0x8, $0xfe00", options(att_syntax));
    }
}

#[inline(never)]
fn get_mmap() {
    let mut contid: u32 = 0;
    let mut entries: u32 = 0;
    let mut _signature: u32 = 0;
    let mut _bytes: u32 = 0;

    loop {
        unsafe {
            asm!(
                "int 0x15",
                inout("eax") 0xE820 => _signature,
                inout("ecx") 24 => _bytes,
                inout("ebx") contid,
                in("edx") 0x534D4150,
                in("edi") &mut BOOT.mmap.entries[entries as usize] as *mut MemoryMapEntry,
            );
        }

        if entries >= 32 {
            break;
        } else {
            entries += 1;
        }

        if contid == 0 {
            break;
        }
    }
}

#[inline(never)]
fn get_rsdp() -> Rsdp {
    let mut addr = 0xE0000 as *const u8;
    let end = 0xFFFFF as *const u8;

    unsafe {
        while addr <= end {
            let sig = core::slice::from_raw_parts(addr, 8);
            if sig == b"RSD PTR " {
                let rsdp = (addr as *const Rsdp).read();
                return rsdp;
            }
            addr = addr.add(16);
        }
    }

    Rsdp {
        signature: [0; 8],
        checksum: 0,
        oem_id: [0; 6],
        revision: 0,
        rsdt_address: 0,
    }
}

#[inline(never)]
fn load_vbe_mode(mode: u16) {
    let mode_info_ptr = unsafe { core::ptr::addr_of!(VBE_MODE) as usize };

    unsafe {
        asm!(
            "int 0x10",
            in("ax") 0x4F01,
            in("cx") mode,
            in("edi") mode_info_ptr,
            options(nostack)
        );
    }
}

#[inline(never)]
fn save_vbe_mode(mode: u16) {
    let mode_info_ptr = unsafe { &raw mut BOOT.mode as *mut VbeModeInfoBlock as usize };

    unsafe {
        asm!(
             "int 0x10",
             in("ax") 0x4F01,
             in("cx") mode,
             in("edi") mode_info_ptr,
             options(nostack)
        );
    }
}

#[inline(never)]
fn get_vbe_info() -> VbeInfoBlock {
    let mut vbe_info = unsafe { core::mem::zeroed() };

    unsafe {
        asm!(
            "int 0x10",
            in("ax") 0x4F00,
            in("edi") ((&mut vbe_info as *mut VbeInfoBlock as usize)),
            options(nostack)
        );
    }

    vbe_info
}

#[inline(never)]
fn find_vbe_mode() -> u16 {
    let base_mode = unsafe { BOOT.vbe.video_ptr } as *const u16;

    let mut best_mode = 0x0013;
    let mut i = 0;
    let mut mode;

    loop {
        mode = unsafe { core::ptr::read_volatile(base_mode.offset(i)) };

        if mode == 0xFFFF {
            break;
        }

        load_vbe_mode(mode);
        let mode_width = unsafe { VBE_MODE.width };
        let mode_height = unsafe { VBE_MODE.height };
        let mode_bpp = unsafe { VBE_MODE.bpp };
        let mode_red = unsafe { VBE_MODE.red_field_position };
        let mode_green = unsafe { VBE_MODE.green_field_position };
        let mode_blue = unsafe { VBE_MODE.blue_field_position};
        let mode_attr = unsafe { VBE_MODE.attributes };

        
        if mode_red != 16 || mode_green != 8 || mode_blue != 0 {
            i += 1;
            continue;
        }

        load_vbe_mode(best_mode);
        let best_mode_width = unsafe { VBE_MODE.width };
        let best_mode_height = unsafe { VBE_MODE.height };

        if (mode_width > best_mode_width
            && mode_width <= 1024
            && mode_height > best_mode_height
            && mode_height <= 1024
            && (mode_bpp == 24 || mode_bpp == 32))
            && (mode_attr & 0x80) != 0
        {
            best_mode = mode;
            save_vbe_mode(best_mode);
        }

        i += 1;
    }

    best_mode
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