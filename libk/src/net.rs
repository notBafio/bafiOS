use alloc::{Layout, alloc, dealloc};
use core::ptr;

pub struct Socket {
    pub port: u16,
    pub buffer: u32,
}

const BUFFER_SIZE: usize = 1500;

impl Socket {
    pub fn new(port: u16) -> Self {
        unsafe {
            if !(*(&raw mut NET)).is_inited() {
                (*(&raw mut NET)).init();
            }
        }

        let layout = Layout::from_size_align(BUFFER_SIZE, 8).expect("Invalid layout");

        let ptr = unsafe { alloc::alloc(layout) };

        if ptr.is_null() {
            panic!("Memory allocation failed");
        }

        unsafe {
            ptr::write_bytes(ptr, 0, BUFFER_SIZE);
        }

        crate::syscall::syscall(31, port as u32, ptr as u32, 0);

        Socket {
            port,
            buffer: ptr as u32,
        }
    }

    pub fn close(&self) {
        let layout = Layout::from_size_align(BUFFER_SIZE, 8).expect("Invalid layout");

        crate::syscall::syscall(32, self.buffer, 0, 0);

        unsafe {
            alloc::dealloc(self.buffer as *mut u8, layout);
        }
    }

    pub fn send(&self, data: &[u8]) {
        crate::syscall::syscall(30, data.as_ptr() as u32, data.len() as u32, 0);
    }

    pub fn recv(&self, mut len: usize) -> &[u8] {
        if len > BUFFER_SIZE {
            len = BUFFER_SIZE;
        }

        unsafe {
            loop {
                if *(self.buffer as *const u8) != 0 {
                    break;
                }
            }

            let packet = core::slice::from_raw_parts(self.buffer as *const u8, len);
            core::ptr::write(self.buffer as *mut u32, 0);

            return packet;
        }
    }
}

#[derive(Debug)]
pub struct Net {
    pub mac_address: [u8; 6],
    pub ip: [u8; 4],
    pub subnet: [u8; 4],
    pub gateway: [u8; 4],
    pub dns: [u8; 4],
}

pub static mut NET: Net = Net {
    mac_address: [0; 6],
    ip: [0; 4],
    subnet: [0; 4],
    gateway: [0; 4],
    dns: [0; 4],
};

impl Net {
    pub fn is_inited(&self) -> bool {
        if self.mac_address == [0; 6] {
            false
        } else {
            true
        }
    }

    pub fn init(&mut self) {
        let header = unsafe {
            *(crate::syscall::syscall(33, 0, 0, 0) as *const crate::packets::Rtl8139Driver)
        };

        self.mac_address = header.mac_address;
        self.ip = header.ip;
        self.subnet = header.subnet;
        self.gateway = header.gateway;
        self.dns = header.dns;

        crate::println!("{:?}", self);
    }
}
