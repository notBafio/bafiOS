use libk::port::{inb, outb};
use libk::println;

pub fn init() {
    outb(0x64, 0xA8);
    wait();
    outb(0x64, 0x20);
    wait_input();
    let mut status = inb(0x60);
    status |= 0b11;

    outb(0x64, 0x60);
    wait();
    outb(0x60, status);
    mouse_write(0xF6);
    let _ack1 = mouse_read();

    mouse_write(0xF4);
    let _ack2 = mouse_read();
}

fn mouse_write(value: u8) {
    wait();
    outb(0x64, 0xD4);
    wait();
    outb(0x60, value);
}

fn mouse_read() -> u8 {
    wait_input();
    let response = inb(0x60);
    if response != 0xFA {
        println!("Mouse did not acknowledge: {:#X}", response);
    }

    response
}

fn wait() {
    let mut time = 100_000;

    while time > 1 {
        if (inb(0x64) & 0b10) == 0 {
            return;
        }

        time -= 1;
    }
}

fn wait_input() {
    let mut time = 100_000;

    while time > 1 {
        if (inb(0x64) & 0b1) == 1 {
            return;
        }

        time -= 1;
    }
}
