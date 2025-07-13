use core::arch::asm;

pub fn read<T>(lba: u64, sectors: u16, target: *mut T) {
    while is_busy() {}

    outb(0x3f6, 0b00000010);

    outb(0x1F1, 0x00);
    outb(0x1F2, sectors as u8);
    outb(0x1F3, lba as u8);
    outb(0x1F4, (lba >> 8) as u8);
    outb(0x1F5, (lba >> 16) as u8);
    outb(0x1F6, (0xE0 | ((lba >> 24) & 0x0F)) as u8);

    outb(0x1F7, 0x20);

    let mut sectors_left = sectors;
    let mut target_pointer = target;

    while sectors_left > 0 {
        for _ in 0..256 {
            while is_busy() {}
            while !is_ready() {}

            let bytes_16 = inw(0x1F0);

            unsafe {
                core::ptr::write_unaligned(target_pointer as *mut u8, (bytes_16 & 0xFF) as u8);
                target_pointer = target_pointer.byte_add(1);
                core::ptr::write_unaligned(
                    target_pointer as *mut u8,
                    ((bytes_16 >> 8) & 0xFF) as u8,
                );
                target_pointer = target_pointer.byte_add(1);
            }
        }
        sectors_left -= 1;
    }

    reset();
}

pub fn reset() {
    outb(0x3f6, 0b00000110);
    outb(0x3f6, 0b00000010);
}

pub fn is_ready() -> bool {
    let status: u8 = inb(0x1F7);

    (status & 0b01000000) != 0
}

pub fn is_busy() -> bool {
    let status: u8 = inb(0x1F7);

    (status & 0b10000000) != 0
}

pub fn inb(port: u16) -> u8 {
    let value: u8;
    unsafe {
        asm!(
            "in al, dx",
            out("al") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags));
    }
    value
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

pub fn inw(port: u16) -> u16 {
    let value: u16;
    unsafe {
        asm!(
            "in ax, dx",
            out("ax") value,
            in("dx") port,
            options(nomem, nostack, preserves_flags)
        );
    }
    value
}
