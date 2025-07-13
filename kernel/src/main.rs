#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

extern crate alloc;

mod ac97;
mod acpi;
mod composer;
mod disk;
mod display;
mod dma;
mod elf;
mod exceptions;
mod idt;
mod heap;
mod keyboard;
mod mouse;
mod net;
mod pci;
mod pic;
mod pmm;
mod task;
mod tss;
mod fs;

use libk;

use core::arch::asm;
use core::panic::PanicInfo;
use idt::IDT;
use pic::PICS;

#[global_allocator]
static ALLOC: heap::Allocator = heap::Allocator::new();

pub static mut BOOTINFO: libk::boot::BootInfo = libk::boot::BOOTINFO_NULL;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".start")]
pub extern "C" fn _start() -> ! {

    ALLOC.init();

    unsafe {
        BOOTINFO = args();
    }

    tss_flush();

    unsafe {
        dma::init();
        (*(&raw mut pmm::PADDR)).init();
        (*(&raw mut composer::DISPLAY_SERVER)).init();
    }

    libk::println!("[!] Kernel reached and args loaded");

    unsafe {
        (*(&raw mut task::TASK_MANAGER)).lock().init();
        (*(&raw mut task::TASK_MANAGER))
            .lock()
            .add_user_task(test as u32, None);

        idt();
        mouse::init();

        (*(&raw mut net::rtl8139::RTL8139)).init();

        libk::println!("[-] Kernel ended");

        libk::enable_interrupts();

        loop {
            asm!("hlt");
        }
    }
}

fn test() -> ! {
    let _ = libk::elf::load_elf("USER/LOGIN.ELF", None);
    libk::syscall::exit();
}

fn idt() {
    unsafe {
        (*(&raw mut IDT)).init();
        (*(&raw mut IDT)).processor_exceptions();
        (*(&raw mut IDT)).add_ring_3(exceptions::TIMER_INT as usize, task::timer as u32);
        (*(&raw mut IDT)).add(
            exceptions::KEYBOARD_INT as usize,
            exceptions::keyboard_handler as u32,
        );
        (*(&raw mut IDT)).add(exceptions::RTC_INT as usize, exceptions::rtc_handler as u32);
        (*(&raw mut IDT)).add(
            exceptions::MOUSE_INT as usize,
            exceptions::mouse_handler as u32,
        );
        (*(&raw mut IDT)).add(exceptions::NET_INT as usize, net::rtl8139::net as u32);
        (*(&raw mut IDT)).add_ring_3(0x80, exceptions::syscall as u32);
        (*(&raw mut IDT)).load();
        (*(&raw mut PICS)).init();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    libk::println!("PANIC\n {}", info);
    loop {}
}

fn args() -> libk::boot::BootInfo {
    let bx: u16;
    unsafe {
        asm!( "mov {0:x}, bx" , out(reg) bx );
    }
    let info = bx as *const libk::boot::BootInfo;
    unsafe { *info }
}

pub fn set_tss(esp: u32) {
    unsafe {
        let tss = &mut *(BOOTINFO.tss as u32 as *mut tss::TaskStateSegment);
        tss.esp0 = esp;
        tss.ss0 = 0x10;
    }
}

fn tss_flush() {
    unsafe {
        asm!("mov ax, 0x28", "ltr ax",);
    }
}
