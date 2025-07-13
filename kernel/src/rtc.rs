use libk::port::{inb, outb};
use core::arch::asm;

pub static mut CPU_CLOCK: u64 = 0;

const SECONDS: u8 = 0x00;
const MINUTES: u8 = 0x02;
const HOURS: u8 = 0x04;
const DAY: u8 = 0x07;
const MONTH: u8 = 0x08;
const YEAR: u8 = 0x09;

fn read_rtc(reg: u8) -> u8 {
    while updating() {}

    outb(0x70, reg);
    let v = inb(0x71);

    return bcd_to_binary(v);
}

fn bcd_to_binary(bcd: u8) -> u8 {
    ((bcd / 16) * 10) + (bcd % 16)
}

fn updating() -> bool {
    outb(0x70, 0x0A);
    (inb(0x71) & 0x80) != 0
}

pub fn get_date() {

    let s = read_rtc(SECONDS);
    let m = read_rtc(MINUTES);
    let h = read_rtc(HOURS);

    libk::println!("{}:{}:{}", h, m, s);

}