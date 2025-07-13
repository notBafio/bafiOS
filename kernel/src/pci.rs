use libk::port::{inl, outl};
use libk::println;

const PCI_CONFIG_ADDRESS: u32 = 0xCF8;
const PCI_CONFIG_DATA: u32 = 0xCFC;

#[derive(Debug, Copy, Clone)]
pub struct PciDevice {
    class: u32,
    subclass: u32,
    vendor_id: u32,
    device_id: u32,
    bus: u8,
    device: u8,
    function: u8,
}

fn pci_config_address(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000
}

fn pci_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let mut address: u32 = 0x80000000;
    address |= ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((function as u32) << 8)
        | ((offset as u32) & 0xFC);
    outl(0xCF8, address);
    let value = inl(0xCFC);

    value
}

pub fn find_device(v_id: u32, d_id: u32) -> Option<PciDevice> {
    for bus in 0..=255 {
        for device in 0..32 {
            for function in 0..8 {
                let vendor_id = pci_read(bus, device, function, 0) & 0xFFFF;

                if vendor_id != 0xFFFF {
                    let device_id = pci_read(bus, device, function, 2) >> 16;
                    let class_subclass = pci_read(bus, device, function, 8);
                    let class = (class_subclass >> 24) & 0xFF;
                    let subclass: u32 = (class_subclass >> 16) & 0xFF;

                    if vendor_id == v_id && device_id == d_id {
                        return Some(PciDevice {
                            class,
                            subclass,
                            vendor_id,
                            device_id,
                            bus,
                            device,
                            function,
                        });
                    }
                }
            }
        }
    }
    None
}

pub fn list_devices() {
    for bus in 0..=255 {
        for device in 0..32 {
            for function in 0..8 {
                let vendor_id = pci_read(bus, device, function, 0) & 0xFFFF;

                if vendor_id != 0xFFFF {
                    let device_id = (pci_read(bus, device, function, 0) >> 16) & 0xFFFF;
                    let class_subclass = pci_read(bus, device, function, 8);
                    let class_code = (class_subclass >> 24) & 0xFF;
                    let subclass_code = (class_subclass >> 16) & 0xFF;

                    libk::println!(
                        "CLASS: {:#X}, SUBCLASS: {:#X}, VENDOR: {:#X}, DEVICE: {:#X}",
                        class_code,
                        subclass_code,
                        vendor_id,
                        device_id
                    );
                }
            }
        }
    }
}
impl PciDevice {
    pub fn new(vendor_id: u32, device_id: u32) -> Option<PciDevice> {
        println!("Finding device...");
        find_device(vendor_id, device_id)
    }

    fn get_pci_irq(bus: u8, device: u8, function: u8) -> u8 {
        let value = pci_read(bus, device, function, 0x3C);
        (value & 0xFF) as u8
    }

    pub fn get_bar(&self, bar_index: u8) -> Option<u32> {
        if bar_index > 5 {
            return None;
        }

        let bar_offset = 0x10 + (bar_index as u32 * 4);

        let bar_value = self.read_config_register(bar_offset);

        if bar_value == 0 {
            return None;
        }

        let is_io = (bar_value & 0x1) == 1;

        if is_io {
            Some(bar_value & !0x3)
        } else {
            Some(bar_value & !0xF)
        }
    }

    fn read_config_register(&self, offset: u32) -> u32 {
        let address = self.get_config_address(offset);

        let config_addr_port = 0xCF8;
        let config_data_port = 0xCFC;

        libk::port::outl(config_addr_port, address);

        libk::port::inl(config_data_port)
    }

    fn get_config_address(&self, offset: u32) -> u32 {
        let enable_bit = 1 << 31;
        let bus = (self.bus as u32) << 16;
        let device = (self.device as u32) << 11;
        let function = (self.function as u32) << 8;
        let aligned_offset = offset & 0xFC;

        enable_bit | bus | device | function | aligned_offset
    }

    pub fn enable_bus_mastering(&self) -> bool {
        let current_command = match self.read_command_register() {
            Some(cmd) => cmd,
            None => return false,
        };

        let new_command = current_command | 0x0004 | 0x0002 | 0x0001;

        self.write_command_register(new_command);

        match self.read_command_register() {
            Some(cmd) => (cmd & 0x0004) != 0,
            None => false,
        }
    }

    fn read_command_register(&self) -> Option<u16> {
        let config_addr = self.generate_config_address(0x04);

        libk::port::outl(0xCF8, config_addr);

        let value = libk::port::inl(0xCFC) & 0xFFFF;
        Some(value as u16)
    }

    fn write_command_register(&self, value: u16) {
        let config_addr = self.generate_config_address(0x04);

        libk::port::outl(0xCF8, config_addr);

        let current = libk::port::inl(0xCFC);

        let new_value = (current & 0xFFFF0000) | (value as u32);

        libk::port::outl(0xCFC, new_value);
    }

    fn generate_config_address(&self, register: u8) -> u32 {
        let enable_bit: u32 = 1 << 31;
        let bus: u32 = (self.bus as u32) << 16;
        let device: u32 = (self.device as u32) << 11;
        let function: u32 = (self.function as u32) << 8;
        let register: u32 = (register as u32) & 0xFC;

        enable_bit | bus | device | function | register
    }
}
