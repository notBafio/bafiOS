use crate::exceptions;
use core::arch::asm;
use core::mem::size_of;

pub static mut IDT: Idt = Idt {
    entries: unsafe { core::mem::zeroed() },
};

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Entry {
    offset_low: u16,
    segment_selector: u16,
    reserved: u8,
    flags: u8,
    offset_high: u16,
}

impl Entry {
    pub fn set(&mut self, offset: u32) {
        self.segment_selector = 0x08;
        self.offset_low = ((offset << 16) >> 16) as u16;
        self.offset_high = (offset >> 16) as u16;
        self.flags = 0x8E;
    }

    pub fn set_ring_3(&mut self, offset: u32) {
        self.segment_selector = 0x08;
        self.offset_low = ((offset << 16) >> 16) as u16;
        self.offset_high = (offset >> 16) as u16;
        self.flags = 0xEE;
    }
}

#[repr(C, packed)]
pub struct Idt {
    entries: [Entry; 256],
}

#[repr(C, packed)]
pub struct Descriptor {
    size: u16,
    offset: *const Idt,
}

impl Idt {
    pub fn init(&mut self) {
        for i in 0..self.entries.len() {
            self.add(i, exceptions::generic_handler as u32);
        }
    }

    pub fn add(&mut self, int: usize, handler: u32) {
        self.entries[int].set(handler);
    }

    pub fn add_ring_3(&mut self, int: usize, handler: u32) {
        self.entries[int].set_ring_3(handler);
    }

    pub fn load(&self) {
        let idt_descriptor = Descriptor {
            size: (256 * size_of::<Entry>() - 1) as u16,
            offset: self,
        };

        unsafe {
            asm!("lidt [{0:e}]", in(reg) &idt_descriptor);
        }
    }

    pub fn processor_exceptions(&mut self) {
        self.add(0x0, exceptions::div_error as u32);
        self.add(0x5, exceptions::bounds as u32);
        self.add(0x6, exceptions::invalid_opcode as u32);
        self.add(0x8, exceptions::double_fault as u32);
        self.add(0xd, exceptions::general_protection_fault as u32);
        self.add(0xe, exceptions::page_fault as u32);
    }
}

pub fn interrupts() -> bool {
    let flags: u32;

    unsafe {
        asm!(
            "pushfd",
            "pop {}",
            out(reg) flags,
            options(nomem, nostack, preserves_flags)
        );
    }
    (flags & (1 << 9)) != 0
}
