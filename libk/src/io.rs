use crate::alloc::string::String;
use crate::syscall::malloc;
use alloc::vec::Vec;

#[derive(Clone, Debug)]
pub struct File {
    pub fname: String,
    pub size: u32,
    pub ptr: u32,
    pub read: bool,
    pub write: bool,
}

pub fn make_file(fname: &str) {
    let string_ptr = fname.as_ptr();
    let string_len = fname.len();

    crate::syscall::syscall(40, string_ptr as u32, 0, string_len as u32);
}

pub fn make_dir(fname: &str) {
    let string_ptr = fname.as_ptr();
    let string_len = fname.len();

    crate::syscall::syscall(42, string_ptr as u32, 0, string_len as u32);
}
impl File {
    pub fn new(fname: &str) -> File {
        let size = size(fname);

        if size == 699669 {
            crate::println!("[x] File not found");
            let string_ptr = fname.as_ptr();
            let string_len = fname.len();

            crate::syscall::syscall(40, string_ptr as u32, 0, string_len as u32);

            return File {
                fname: String::from(fname),
                size: 0,
                ptr: 0,
                read: false,
                write: false,
            };
        } else {
            let addr = malloc(size);

            let string_ptr = fname.as_ptr();
            let string_len = fname.len();

            crate::syscall::syscall(2, string_ptr as u32, addr, string_len as u32);

            return File {
                fname: String::from(fname),
                size,
                ptr: addr,
                read: false,
                write: false,
            };
        }
    }

    pub fn read_bytes(&mut self) -> &[u8] {
        crate::println!("{:?}", self);

        if self.ptr == 0 {
            self.size = size(&self.fname);

            if self.size == 699669 {
                return &[69];
            }

            self.ptr = malloc(self.size);

            let string_ptr = self.fname.as_ptr();
            let string_len = self.fname.len();

            crate::syscall::syscall(2, string_ptr as u32, self.ptr, string_len as u32);

            return unsafe {
                core::slice::from_raw_parts(self.ptr as *const u8, self.size as usize)
            };
        } else {
            return unsafe {
                core::slice::from_raw_parts(self.ptr as *const u8, self.size as usize)
            };
        }
    }

    pub fn read_to_buffer(&self, buffer: u32) {
        let string_ptr = self.fname.as_ptr();
        let string_len = self.fname.len();

        crate::syscall::syscall(2, string_ptr as u32, buffer, string_len as u32);
    }

    pub fn write(&self, data: &[u8]) {
        let string_ptr = self.fname.as_ptr();
        let string_len = self.fname.len();

        let data_ref = (data.as_ptr() as u32, data.len() as u32);

        crate::syscall::syscall(
            38,
            string_ptr as u32,
            &data_ref as *const (u32, u32) as u32,
            string_len as u32,
        );
    }

    pub fn append(&self, data: &[u8]) {
        let string_ptr = self.fname.as_ptr();
        let string_len = self.fname.len();

        let data_ref = (data.as_ptr() as u32, data.len() as u32);

        crate::syscall::syscall(
            39,
            string_ptr as u32,
            &data_ref as *const (u32, u32) as u32,
            string_len as u32,
        );
    }

    pub fn close(&self) {
        crate::syscall::free(self.ptr);
    }

    pub fn get_file_entry(&self) -> Entry {
        unsafe {
            let e = crate::syscall::syscall(
                3,
                self.fname.as_ptr() as u32,
                core::ptr::addr_of!(FILE_ENTRY) as u32,
                self.fname.len() as u32,
            );

            FILE_ENTRY = *(e as *const Entry);
        }

        unsafe { (*(&raw mut FILE_ENTRY)).clone() }
    }

    pub fn is_dir(&self) -> bool {
        let file_entry = self.get_file_entry();
        file_entry.attributes & 0x10 != 0
    }

    pub fn get_file_extention(&self) -> &str {
        &self.fname[self.fname.len() - 3..]
    }
}

pub static mut FILE_ENTRY: Entry = Entry {
    name: [0; 11],
    attributes: 0,
    reserved: 0,
    created_time_tenths: 0,
    created_time: 0,
    created_date: 0,
    accessed_date: 0,
    first_cluster_high: 0,
    modified_time: 0,
    modified_date: 0,
    first_cluster_low: 0,
    size: 0,
};

pub fn size(fname: &str) -> u32 {
    let string_ptr = fname.as_ptr();
    let string_len = fname.len();

    crate::syscall::syscall(4, string_ptr as u32, 0, string_len as u32)
}

pub fn dir_entries(fname: &str) -> u32 {
    let string_ptr = fname.as_ptr();
    let string_len = fname.len();

    crate::syscall::syscall(28, string_ptr as u32, 0, string_len as u32)
}

pub fn get_entry(fname: &str, index: u8) -> Option<Entry> {
    let string_ptr = fname.as_ptr();
    let string_len = fname.len();

    let entry_addr =
        crate::syscall::syscall(29, string_ptr as u32, index as u32, string_len as u32);

    if entry_addr > 0 {
        let entry = unsafe { core::ptr::read(entry_addr as *const Entry) };

        crate::syscall::free(entry_addr);

        return Some(entry);
    }

    None
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct Entry {
    pub name: [u8; 11],
    pub attributes: u8,
    reserved: u8,
    created_time_tenths: u8,
    created_time: u16,
    created_date: u16,
    accessed_date: u16,
    first_cluster_high: u16,
    modified_time: u16,
    modified_date: u16,
    first_cluster_low: u16,
    pub size: u32,
}

impl Entry {
    pub fn is_dir(&self) -> bool {
        self.attributes & 0x10 != 0
    }
}

pub fn expand_path_8_3(path: &str) -> &[u8] {
    assert!(path.len() == 11, "Path must be exactly 11 characters long");

    if !path.contains(' ') {
        return path.as_bytes();
    }

    static mut BUFFER: [u8; 12] = [0; 12];

    let name_part = &path[..8];
    let ext_part = &path[8..];

    let trimmed_name = name_part.trim_end();
    let trimmed_ext = ext_part.trim_end();

    unsafe {
        let mut idx = 0;

        for byte in trimmed_name.bytes() {
            BUFFER[idx] = byte;
            idx += 1;
        }

        if !trimmed_ext.is_empty() {
            BUFFER[idx] = b'.';
            idx += 1;

            for byte in trimmed_ext.bytes() {
                BUFFER[idx] = byte;
                idx += 1;
            }
        }

        &BUFFER[..idx]
    }
}
