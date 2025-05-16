#![no_std]

extern crate alloc;

pub mod boot;
pub mod elf;
pub mod hashmap;
pub mod io;
pub mod heap;
pub mod mmio;
pub mod mutex;
pub mod net;
pub mod packets;
pub mod port;
pub mod rng;
pub mod serial;
pub mod syscall;
pub mod hash;

#[inline(always)]
pub fn disable_interrupts() {
    unsafe {
        core::arch::asm!("cli", options(nomem, nostack));
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    unsafe {
        core::arch::asm!("sti", options(nomem, nostack));
    }
}
