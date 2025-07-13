use core::arch::asm;

#[repr(C, packed)]
struct Dap {
    size: u8,
    zero: u8,
    sectors: u16,
    offset: u16,
    segment: u16,
    lba: u64,
}

pub fn read_stub() {
    let disk_setup = Dap {
        size: core::mem::size_of::<Dap>() as u8,
        zero: 0,
        sectors: 32,
        offset: 0x7E00,
        segment: 0,
        lba: 2048,
    };

    unsafe {
        asm!(
            "mov {1:x}, si",
            "mov si, {0:x}",
            "int 0x13",

            "jc fail",
            "mov si, {1:x}",

            in(reg) &disk_setup as *const Dap as u16,
            out(reg) _,
            in("ax") 0x4200 as u16,
            in("dx") 0x80 as u16,
        );
    }
}
