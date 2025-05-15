use crate::dma::*;
use core::arch::asm;
use libk::port::{inb, inw, outb, outl, outw};

pub fn read<T>(lba: u64, sectors: u16, disk: u8, target: *mut T) {
    while is_busy() {}

    outb(0x3f6, 0b00000010);

    let disk = disk;

    outb(0x1F1, 0x00);
    outb(0x1F2, sectors as u8);
    outb(0x1F3, lba as u8);
    outb(0x1F4, (lba >> 8) as u8);
    outb(0x1F5, (lba >> 16) as u8);

    outb(0x1F6, (disk as u64 | ((lba >> 24) & 0x0F)) as u8);

    outb(0x1F7, 0x20);

    let mut sectors_left = sectors;
    let mut target_pointer = target;

    while sectors_left > 0 {
        for _ in 0..256 {
            while is_busy() {}
            while !is_ready() {}

            let bytes_16 = inw(0x1F0) as u16;

            unsafe {
                core::ptr::write(target_pointer as *mut u16, bytes_16);
                target_pointer = target_pointer.byte_add(2);
            }
        }
        sectors_left -= 1;
    }

    reset();
}

pub fn write<T>(lba: u64, content: *const T, length_bytes: usize) {

    let total_sectors = (length_bytes + 511) / 512;
    let mut offset: usize = 0;

    while offset < total_sectors {

        let sectors_to_write = if (total_sectors - offset) > 255 {
            255
        } else {
            (total_sectors - offset) as u16
        };

        while is_busy() {}

        outb(0x3f6, 0b00000010);
        outb(0x1F1, 0x00);
        outb(0x1F2, sectors_to_write as u8);
        outb(0x1F3, (lba + offset as u64) as u8);
        outb(0x1F4, ((lba + offset as u64) >> 8) as u8);
        outb(0x1F5, ((lba + offset as u64) >> 16) as u8);
        outb(0x1F6, (0xE0 | (((lba + offset as u64) >> 24) & 0x0F)) as u8);
        outb(0x1F7, 0x30);

        let content_ptr = content as *const u8;
        let mut sectors_left = sectors_to_write;

        while sectors_left > 0 {
            let bytes_remaining = length_bytes.saturating_sub(offset * 512);

            for i in 0..256 {
                while is_busy() {}
                while !is_ready() {}

                let byte_offset = offset * 512 + i * 2;
                let mut b_2_w: u16 = 0;

                if byte_offset < length_bytes {
                    b_2_w = unsafe { *content_ptr.add(byte_offset) as u16 };
                }

                if byte_offset + 1 < length_bytes {
                    b_2_w |= (unsafe { *content_ptr.add(byte_offset + 1) as u16 }) << 8;
                }

                outw(0x1F0, b_2_w);
            }
            sectors_left -= 1;
            offset += 1;
        }

        reset();
        outb(0x1F7, 0xE7);
    }
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

fn delay() {
    for _ in 0..10000 {
        unsafe { asm!("nop") };
    }
}

pub fn check_disk() -> [bool; 2] {
    let mut master = false;
    let mut slave = false;

    outb(0x1F6, 0xF0);
    outb(0x1F7, 0xEC);

    delay();

    let status = inb(0x1F7);
    if status != 0 {
        slave = true;
    }

    delay();

    outb(0x1F6, 0xE0);
    outb(0x1F7, 0xEC);

    delay();

    let status = inb(0x1F7);
    if status != 0 {
        master = true;
    }

    [master, slave]
}
