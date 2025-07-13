use crate::pci::*;
use libk::port::*;
use libk::println;

fn test_pci_detection() {
    unsafe {
        if let Some(dev) = find_device(0x8086, 0x7010) {
            println!("IDE controller found!");

            BM_BASE = dev.get_bar(4).unwrap() as u16;
            BMR_COMMAND = BM_BASE + 0;
            BMR_STATUS = BM_BASE + 2;
            BMR_PRDT = BM_BASE + 4;

            dev.enable_bus_mastering();
        } else {
            panic!("PIIX4 controller not found!");
        }
    }
}

struct PrdtEntry {
    buffer_phys: u32,
    transfer_size: u16,
    flags: u16,
}

static mut PRDT: PrdtEntry = PrdtEntry {
    buffer_phys: 0,
    transfer_size: 0,
    flags: 0,
};

static mut BM_BASE: u16 = 0;
static mut BMR_COMMAND: u16 = unsafe { BM_BASE + 0 };
static mut BMR_STATUS: u16 = unsafe { BM_BASE + 2 };
static mut BMR_PRDT: u16 = unsafe { BM_BASE + 4 };

const ATA: u16 = 0x1F0;
const ATA_DISK: u16 = ATA + 6;
const ATA_SECTOR: u16 = ATA + 2;
const ATA_LBA_LOW: u16 = ATA + 3;
const ATA_LBA_MID: u16 = ATA + 4;
const ATA_LBA_HIG: u16 = ATA + 5;
const ATA_COMMAND: u16 = ATA + 7;

const ATA_READ_DMA: u8 = 0xC8;
const ATA_WRITE_DMA: u8 = 0xCA;

pub fn read<T>(lba: u64, sectors: u8, disk: u8, target: *mut T) {
    unsafe {
        PRDT.buffer_phys = target as u32;
        PRDT.transfer_size = 512 * sectors as u16;
        PRDT.flags = 0x8000;

        outb(BMR_COMMAND, 0);

        outl(BMR_PRDT, core::ptr::addr_of!(PRDT) as u32);

        outb(ATA_DISK, (disk as u64 | ((lba >> 24) & 0x0F)) as u8);

        outb(ATA_SECTOR, sectors as u8);
        outb(ATA_LBA_LOW, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HIG, (lba >> 16) as u8);

        outb(ATA_COMMAND, ATA_READ_DMA);

        outb(BMR_COMMAND, 0x8 | 0x1);

        loop {
            let status = inb(BMR_STATUS);

            if status & 0x02 != 0 {
                panic!("DMA error occurred");
            }

            if (status & 0x04) == 0 {
                break;
            }
        }

        let mut command = inb(BMR_COMMAND);
        command &= !0x01;

        outb(BMR_COMMAND, command);

        outb(BMR_STATUS, 0x04 | 0x02);
    }
}

pub fn write<T>(lba: u64, sectors: u8, disk: u8, buffer: *const T) {
    unsafe {

        outb(BMR_COMMAND, 0);

        PRDT.buffer_phys = buffer as u32;
        PRDT.transfer_size = 512 * sectors as u16;
        PRDT.flags = 0x8000;

        outl(BMR_PRDT, core::ptr::addr_of!(PRDT) as u32);

        outb(ATA_DISK, (disk as u64 | ((lba >> 24) & 0x0F)) as u8);

        outb(ATA_SECTOR, sectors as u8);
        outb(ATA_LBA_LOW, lba as u8);
        outb(ATA_LBA_MID, (lba >> 8) as u8);
        outb(ATA_LBA_HIG, (lba >> 16) as u8);

        outb(ATA_COMMAND, ATA_WRITE_DMA);

        outb(BMR_COMMAND, 0x1);

        loop {
            let status = inb(BMR_STATUS);

            if status & 0x02 != 0 {
                panic!("DMA error occurred");
            }

            if (status & 0x04) == 0 {
                break;
            }
        }

        let mut command = inb(BMR_COMMAND);
        command &= !0x01;


        outb(BMR_COMMAND, command);
    }
}

pub fn init() {
    test_pci_detection();
}
