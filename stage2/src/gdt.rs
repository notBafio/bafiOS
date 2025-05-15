use core::arch::asm;
use core::mem::size_of;
use core::ptr::addr_of;

pub static mut GDT: Gdt = {
    let limit = (0xFFFF << 0) | (0xF << 48);
    let base = (0x0000 << 16) | (0x00 << 56);

    let zero =        Entry { entry: 0, };
    let kernel_code = Entry { entry: limit | base | (0x9A << 40) | (0xC << 52), };
    let kernel_data = Entry { entry: limit | base | (0x92 << 40) | (0xC << 52), };
    let user_code =   Entry { entry: limit | base | (0xFA << 40) | (0xC << 52), };
    let user_data =   Entry { entry: limit | base | (0xF2 << 40) | (0xC << 52), };
    let tss_entry =   Entry { entry: 0, };

    Gdt {
        entries: [
            zero,
            kernel_code,
            kernel_data,
            user_code,
            user_data,
            tss_entry,
        ],
    }
};

#[repr(C, packed)]
pub struct Gdt {
    entries: [Entry; 6],
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Entry {
    entry: u64,
}

#[repr(C, packed)]
pub struct Descriptor {
    size: u16,
    offset: *const Gdt,
}

impl Gdt {
    pub fn load(&self) {
        let gdt_descriptor = Descriptor {
            size: (6 * size_of::<Entry>() - 1) as u16,
            offset: self,
        };

        unsafe {
            asm!("lgdt [{0:e}]", in(reg) &gdt_descriptor);
        }
    }

    pub fn write_tss(&mut self) -> u16 {
        let tss_limit = size_of::<crate::tss::TaskStateSegment>() - 1;
        let tss_limit_high = ((tss_limit >> 16) & 0xFF) as u8;
        let tss_limit_low = (tss_limit & 0xFFFF) as u16;

        let tss_base = addr_of!(crate::tss::TSS) as u32;
        let tss_base_high = ((tss_base >> 16) & 0xFF) as u8;
        let tss_base_low = (tss_base & 0xFFFF) as u16;

        let tss_limit = ((tss_limit_low as u64) << 0) | ((tss_limit_high as u64) << 48);
        let tss_base = ((tss_base_low as u64) << 16) | ((tss_base_high as u64) << 56);

        self.entries[5] = Entry { entry: tss_limit | tss_base | (0x89 << 40) | (0x0 << 52), };

        addr_of!(crate::tss::TSS) as u16
    }
}
