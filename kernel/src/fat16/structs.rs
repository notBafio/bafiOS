use crate::dma;
use libk::{print, println};

pub static mut FAT16: Fat16 = Fat16 {
    header: NULL_HEADER,
};

pub static mut FAT: FatTable = FatTable {
    base: 0,
    fats: [0; 1024],
};

pub static OFFSET_LBA: u64 = 9216;

const ATTR_READ_ONLY: u8 = 0x01;
const ATTR_HIDDEN: u8 = 0x02;
const ATTR_SYSTEM: u8 = 0x04;
const ATTR_VOLUME_ID: u8 = 0x08;
const ATTR_DIRECTORY: u8 = 0x10;
const ATTR_ARCHIVE: u8 = 0x20;

pub struct Fat16 {
    header: Mbr,
}

#[derive(Copy, Clone, Debug)]
pub struct FatTable {
    fats: [u16; 1024],
    base: u64,
}

#[derive(Copy, Clone, Debug)]
#[repr(C, packed)]
pub struct Mbr {
    boot_jmp: [u8; 3],

    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fat_count: u8,
    dir_entries_count: u16,
    total_sectors: u16,
    media_descriptor_type: u8,
    sectors_per_fat: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    large_sector_count: u32,

    drive_number: u8,
    reserved: u8,
    signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    system_id: [u8; 8],
    zero: [u8; 460],
}

static NULL_HEADER: Mbr = Mbr {
    boot_jmp: [0; 3],

    oem_id: [0; 8],
    bytes_per_sector: 0,
    sectors_per_cluster: 0,
    reserved_sectors: 0,
    fat_count: 0,
    dir_entries_count: 0,
    total_sectors: 0,
    media_descriptor_type: 0,
    sectors_per_fat: 0,
    sectors_per_track: 0,
    heads: 0,
    hidden_sectors: 0,
    large_sector_count: 0,

    drive_number: 0,
    reserved: 0,
    signature: 0,
    volume_id: 0,
    volume_label: [0; 11],
    system_id: [0; 8],
    zero: [0; 460],
};

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

pub static NULL_ENTRY: Entry = Entry {
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

impl Fat16 {
    pub fn reload(&mut self) {
        let target =
            unsafe { (*(&raw mut crate::pmm::PADDR)).malloc(core::mem::size_of::<Mbr>() as u32) };

        if target.is_none() {
            return;
        }
        let target = target.unwrap();

        dma::read(OFFSET_LBA + 0, 1, 0xE0, target as *mut Mbr);

        self.header = unsafe { *(target as *const Mbr) };

        unsafe {
            let fat = (*(&raw mut crate::pmm::PADDR)).malloc((2 * 1024) as u32);
            let lba: u64 = self.header.reserved_sectors as u64;

            if fat.is_none() {
                return;
            }
            let fat = fat.unwrap();

            dma::read(OFFSET_LBA + lba, 4, 0xE0, fat as *mut u16);

            for i in 0..1024 {
                FAT.fats[i] = *((fat as *const u16).offset(i as isize));
            }

            FAT.base = 0;
            (*(&raw mut crate::pmm::PADDR)).dealloc(target);
            (*(&raw mut crate::pmm::PADDR)).dealloc(fat);
        }
    }

    pub fn get_fat(&self, index: usize) -> u16 {
        let sector = (index / 256) as u64;
        let lba: u64 = self.header.reserved_sectors as u64;

        unsafe {
            if sector < FAT.base || sector >= FAT.base + 4 {
                let fat = (*(&raw mut crate::pmm::PADDR)).malloc((2 * 1024) as u32);

                if fat.is_none() {
                    return 0;
                }
                let fat = fat.unwrap();

                dma::read(OFFSET_LBA + lba + sector, 4, 0xE0, fat as *mut u16);

                for i in 0..1024 {
                    FAT.fats[i] = *((fat as *const u16).offset(i as isize));
                }

                FAT.base = sector;
                (*(&raw mut crate::pmm::PADDR)).dealloc(fat);
            }

            let offset = index % 1024;
            FAT.fats[offset]
        }
    }

    pub fn read(&self, entry: &Entry, target: *mut u8) {
        let mut next_cluster =
            ((entry.first_cluster_high as u32) << 16) | (entry.first_cluster_low as u32);
        let mut current_target = target;

        let data_lba = (self.header.reserved_sectors as u64
            + (self.header.sectors_per_fat as u64 * self.header.fat_count as u64)
            + ((self.header.dir_entries_count as u64 * 32) / self.header.bytes_per_sector as u64))
            as u64;

        let cluster_size =
            self.header.sectors_per_cluster as usize * self.header.bytes_per_sector as usize;

        while next_cluster >= 0x0002 && next_cluster < 0xFFF0 {
            let lba =
                data_lba + ((next_cluster as u64 - 2) * self.header.sectors_per_cluster as u64);

            dma::read(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                current_target,
            );

            next_cluster = self.get_fat(next_cluster as usize) as u32;

            unsafe {
                current_target = current_target.add(cluster_size);
            }
        }
    }

    pub fn read_file(&self, entry: &str, target: *mut u8) {
        let entry = self.find_entry(entry).unwrap();

        let mut next_cluster =
            ((entry.first_cluster_high as u32) << 16) | (entry.first_cluster_low as u32);
        let mut current_target = target;

        let data_lba = (self.header.reserved_sectors as u64
            + (self.header.sectors_per_fat as u64 * self.header.fat_count as u64)
            + ((self.header.dir_entries_count as u64 * 32) / self.header.bytes_per_sector as u64))
            as u64;

        let cluster_size =
            self.header.sectors_per_cluster as usize * self.header.bytes_per_sector as usize;

        while next_cluster >= 0x0002 && next_cluster < 0xFFF0 {
            let lba =
                data_lba + ((next_cluster as u64 - 2) * self.header.sectors_per_cluster as u64);

            dma::read(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                current_target,
            );

            next_cluster = self.get_fat(next_cluster as usize) as u32;

            unsafe {
                current_target = current_target.add(cluster_size);
            }
        }
    }

    pub fn find_entry(&self, path: &str) -> Option<Entry> {
        let parts = split_path(path);
        let mut start = 0;

        if parts.is_empty() {
            return None;
        }

        if parts[0] == "" {
            start = 1;
        }

        let mut current_entry = match self.find(&self.to_fat_name(parts[start])) {
            Some(entry) => entry,
            None => return None,
        };

        for part in &parts[start + 1..] {
            current_entry = match self.find_2(&current_entry, &self.to_fat_name(part)) {
                Some(entry) => entry,
                None => return None,
            };
        }

        Some(current_entry)
    }

    pub fn find_dir(&self, path: &str) -> Option<Entry> {
        let parts = split_path(path);

        if parts.is_empty() || parts.len() == 1 {
            let base_lba = self.header.reserved_sectors as u32
                + (self.header.fat_count as u32 * self.header.sectors_per_fat as u32);

            return Some(Entry {
                name: [32; 11],
                attributes: ATTR_DIRECTORY,
                reserved: 0,
                created_time_tenths: 0,
                created_time: 0,
                created_date: 0,
                accessed_date: 0,
                first_cluster_high: (base_lba >> 16) as u16,
                modified_time: 0,
                modified_date: 0,
                first_cluster_low: (base_lba & 0xFFFF) as u16,
                size: 0,
            });
        }

        let mut start = 0;
        if parts[0] == "" {
            start = 1;
        }
        if parts.len() - start == 1 {
            return self.find(&self.to_fat_name(parts[start]));
        }
        let mut current_entry = match self.find(&self.to_fat_name(parts[start])) {
            Some(entry) => entry,
            None => return None,
        };
        for part in &parts[start + 1..parts.len() - 1] {
            current_entry = match self.find_2(&current_entry, &self.to_fat_name(part)) {
                Some(entry) => entry,
                None => return None,
            };
        }
        Some(current_entry)
    }

    pub fn to_fat_name(&self, fname: &str) -> [u8; 11] {
        let mut fat_name = [32u8; 11];

        for i in 0..fname.chars().count() {
            if i >= 11 {
                break;
            }

            fat_name[i] = fname.chars().nth(i).unwrap() as u8;
        }

        fat_name
    }

    pub fn path_to_fat_name(&self, path: &str) -> [u8; 11] {
        let parts = split_path(path);
        self.to_fat_name(parts[parts.len() - 1])
    }

    pub fn create_file(&mut self, filename: &str) {
        let fat_name = self.path_to_fat_name(filename);
        let parent_dir = self.find_dir(filename);
        let free_cluster = self.get_cluster_free();

        if parent_dir.is_none() || free_cluster.is_none() {
            return;
        }

        let new_entry = Entry {
            name: fat_name,
            attributes: 0x20,
            first_cluster_low: free_cluster.unwrap() as u16,
            first_cluster_high: (free_cluster.unwrap() >> 16) as u16,
            size: 0,
            created_time: 0,
            modified_time: 0,
            created_date: 0,
            modified_date: 0,
            accessed_date: 0,
            created_time_tenths: 0,
            reserved: 0,
        };

        self.make_file(parent_dir.unwrap(), new_entry);
    }

    pub fn create_dir(&mut self, filename: &str) {
        let fat_name = self.path_to_fat_name(filename);
        let parent_dir = self.find_dir(filename);
        let free_cluster = self.get_cluster_free();

        if parent_dir.is_none() || free_cluster.is_none() {
            return;
        }

        let new_entry = Entry {
            name: fat_name,
            attributes: 0x10,
            first_cluster_low: free_cluster.unwrap() as u16,
            first_cluster_high: (free_cluster.unwrap() >> 16) as u16,
            size: 0,
            created_time: 0,
            modified_time: 0,
            created_date: 0,
            modified_date: 0,
            accessed_date: 0,
            created_time_tenths: 0,
            reserved: 0,
        };

        let f1 = Entry {
            name: [b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',],
            attributes: 0x20,
            first_cluster_low: new_entry.first_cluster_low,
            first_cluster_high: new_entry.first_cluster_high,
            size: 0,
            created_time: 0,
            modified_time: 0,
            created_date: 0,
            modified_date: 0,
            accessed_date: 0,
            created_time_tenths: 0,
            reserved: 0,
        };

        let f2 = Entry {
            name: [b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ', b' ',],
            attributes: 0x10,
            first_cluster_low: parent_dir.unwrap().first_cluster_low,
            first_cluster_high: parent_dir.unwrap().first_cluster_high,
            size: 0,
            created_time: 0,
            modified_time: 0,
            created_date: 0,
            modified_date: 0,
            accessed_date: 0,
            created_time_tenths: 0,
            reserved: 0,
        };

        self.make_file(parent_dir.unwrap(), new_entry);
        self.make_file(new_entry, f1);
        self.make_file(new_entry, f2);
    }

    pub fn get_cluster_free(&self) -> Option<u32> {
        for i in 0..(self.header.sectors_per_fat as u32 * (self.header.bytes_per_sector as u32 / 2))
        {
            let fat = self.get_fat(i as usize);

            if fat == 0 {
                return Some(i as u32);
            }
        }

        None
    }

    pub fn find(&self, filename: &[u8; 11]) -> Option<Entry> {
        let root_dir_lba = (self.header.reserved_sectors as u64
            + (self.header.sectors_per_fat as u64 * self.header.fat_count as u64))
            as u64;
        let root_dir_sectors = ((self.header.dir_entries_count as u64 * 32)
            + (self.header.bytes_per_sector as u64 - 1))
            / self.header.bytes_per_sector as u64;

        let mut dir_buffer = [0u8; 512];

        for sector in 0..root_dir_sectors {
            dma::read(
                OFFSET_LBA + root_dir_lba + sector,
                1,
                0xE0,
                dir_buffer.as_mut_ptr(),
            );

            for i in 0..(self.header.bytes_per_sector / 32) {
                let entry_ptr =
                    unsafe { dir_buffer.as_ptr().add((i * 32) as usize) as *const Entry };
                let entry = unsafe { *entry_ptr };

                if entry.name[0] == 0x00 {
                    break;
                }

                if entry.name[0] != 0xE5 && entry.name == *filename {
                    return Some(entry);
                }
            }
        }

        None
    }

    pub fn find_2(&self, dir_entry: &Entry, filename: &[u8; 11]) -> Option<Entry> {
        let mut current_cluster =
            (dir_entry.first_cluster_high as u32) << 16 | (dir_entry.first_cluster_low as u32);

        while current_cluster >= 0x0002 && current_cluster < 0xFFF0 {
            let data_lba = (self.header.reserved_sectors as u64
                + (self.header.sectors_per_fat as u64 * self.header.fat_count as u64)
                + ((self.header.dir_entries_count as u64 * 32)
                    / self.header.bytes_per_sector as u64)) as u64;

            let cluster_lba =
                data_lba + ((current_cluster as u64 - 2) * self.header.sectors_per_cluster as u64);
            let cluster_size = self.header.sectors_per_cluster as usize;

            let mut cluster_buffer = [0u8; 512 * 64];
            dma::read(
                OFFSET_LBA + cluster_lba,
                cluster_size as u8,
                0xE0,
                cluster_buffer.as_mut_ptr(),
            );

            for i in 0..(cluster_size * self.header.bytes_per_sector as usize / 32) {
                let entry_ptr = unsafe { cluster_buffer.as_ptr().add(i * 32) as *const Entry };
                let entry = unsafe { *entry_ptr };

                if entry.name[0] == 0x00 {
                    break;
                }

                if entry.name[0] != 0xE5 && entry.name == *filename {
                    return Some(entry);
                }
            }

            current_cluster = self.get_fat(current_cluster as usize) as u32;
        }

        None
    }

    pub fn append_to_file(&mut self, entry: &str, data: &[u8]) -> Result<(), &'static str> {
        let entry_p = &mut self.find_entry(entry).unwrap();
        let cluster_size =
            self.header.sectors_per_cluster as usize * self.header.bytes_per_sector as usize;
        let data_len = data.len() + 1;
        let mut remaining_data = data_len;
        let mut data_ptr = data.as_ptr();

        let mut current_cluster =
            ((entry_p.first_cluster_high as u32) << 16) | (entry_p.first_cluster_low as u32);
        while self.get_fat(current_cluster as usize) >= 0x0002
            && self.get_fat(current_cluster as usize) < 0xFFF0
        {
            current_cluster = self.get_fat(current_cluster as usize) as u32;
        }

        let mut offset_in_cluster = entry_p.size as usize % cluster_size;

        if offset_in_cluster == 0 {
            let new_cluster = self
                .get_cluster_free()
                .ok_or("No free clusters available")?;
            self.set_fat(current_cluster as usize, new_cluster as u16);
            self.set_fat(new_cluster as usize, 0xFFF8);
            current_cluster = new_cluster;
            offset_in_cluster = 0;
        }

        while remaining_data > 0 {
            let lba = self.cluster_to_lba(current_cluster);
            let cluster_buffer = unsafe {
                (*(&raw mut crate::pmm::PADDR))
                    .malloc(cluster_size as u32)
                    .unwrap()
            };
            dma::read(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                cluster_buffer as *mut u8,
            );

            let bytes_to_write = core::cmp::min(cluster_size - offset_in_cluster, remaining_data);

            unsafe {
                core::ptr::copy(
                    data_ptr,
                    (cluster_buffer as *mut u8).add(offset_in_cluster),
                    bytes_to_write,
                );
            }

            dma::write(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                cluster_buffer as *const u8,
            );

            remaining_data -= bytes_to_write;
            data_ptr = unsafe { data_ptr.add(bytes_to_write) };
            offset_in_cluster = 0;

            if remaining_data > 0 {
                let new_cluster = self
                    .get_cluster_free()
                    .ok_or("No free clusters available")?;
                self.set_fat(current_cluster as usize, new_cluster as u16);
                self.set_fat(new_cluster as usize, 0xFFF8);
                current_cluster = new_cluster;
            }

            unsafe {
                (*(&raw mut crate::pmm::PADDR)).dealloc(cluster_buffer);
            }
        }

        entry_p.size += data_len as u32;

        self.update_directory_entry(entry, entry_p);

        Ok(())
    }

    pub fn count_entries_in_dir(&self, dir_entry: &str) -> u32 {
        let mut count = 0;

        let mut entries = [NULL_ENTRY; 16];
        let target = &mut entries as *mut Entry;

        let dir_entry = self.find_entry(dir_entry);

        let (mut start_lba, sectors_to_read) = if let Some(dir) = dir_entry {
            if dir.attributes & ATTR_DIRECTORY == 0 {
                return 0;
            }

            let mut current_cluster = dir.first_cluster_low;
            let mut sectors_count = 0;

            while current_cluster >= 0x0002 && current_cluster < 0xFFF0 {
                sectors_count += self.header.sectors_per_cluster as u16;
                current_cluster = self.get_fat(current_cluster as usize);
            }

            (
                self.cluster_to_lba(dir.first_cluster_low as u32),
                sectors_count,
            )
        } else {
            let root_dir_lba = self.header.reserved_sectors as u64
                + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64);
            let root_dir_sectors =
                ((self.header.dir_entries_count * 32) + self.header.bytes_per_sector - 1)
                    / self.header.bytes_per_sector;
            (root_dir_lba, root_dir_sectors)
        };

        let mut current_sector = 0;
        let mut current_cluster = dir_entry.map(|e| e.first_cluster_low);

        while current_sector < sectors_to_read {
            dma::read(
                OFFSET_LBA + start_lba + current_sector as u64,
                1,
                0xE0,
                target,
            );

            for entry in entries.iter() {
                if entry.name[0] == 0 {
                    break;
                }

                if entry.name[0] != 0xE5 && entry.attributes & ATTR_VOLUME_ID == 0 {
                    if !(entry.name[0] == b'.' && (entry.name[1] == b' ' || entry.name[1] == b'.'))
                    {
                        count += 1;
                    }
                }
            }

            current_sector += 1;

            if let Some(cluster) = current_cluster {
                let sectors_per_cluster = self.header.sectors_per_cluster as u16;
                if current_sector % sectors_per_cluster == 0 {
                    let next_cluster = self.get_fat(cluster as usize);
                    if next_cluster >= 0x0002 && next_cluster < 0xFFF0 {
                        current_cluster = Some(next_cluster);
                        start_lba = self.cluster_to_lba(next_cluster as u32);
                        current_sector = 0;
                    }
                }
            }
        }

        return count;
    }

    pub fn get_entries_by_id(&self, dir_entry: &str, idx: u8) -> Option<Entry> {
        let mut count = 0;
        let mut idx_entry: Option<Entry> = None;

        let mut entries = [NULL_ENTRY; 16];
        let target = &mut entries as *mut Entry;

        let dir_entry = if dir_entry.is_empty() || dir_entry == "/" {
            None
        } else {
            self.find_entry(dir_entry)
        };

        let (mut start_lba, sectors_to_read) = if let Some(dir) = dir_entry {
            if dir.attributes & ATTR_DIRECTORY == 0 {
                return None;
            }

            let mut current_cluster = dir.first_cluster_low;
            let mut sectors_count = 0;

            while current_cluster >= 0x0000 && current_cluster < 0xFFF0 {
                sectors_count += self.header.sectors_per_cluster as u16;
                current_cluster = self.get_fat(current_cluster as usize);
            }

            (
                self.cluster_to_lba(dir.first_cluster_low as u32),
                sectors_count,
            )
        } else {
            let root_dir_lba = self.header.reserved_sectors as u64
                + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64);
            let root_dir_sectors =
                ((self.header.dir_entries_count * 32) + self.header.bytes_per_sector - 1)
                    / self.header.bytes_per_sector;
            (root_dir_lba, root_dir_sectors)
        };

        let mut current_sector = 0;
        let mut current_cluster = dir_entry.map(|e| e.first_cluster_low);

        while current_sector < sectors_to_read {
            dma::read(
                OFFSET_LBA + start_lba + current_sector as u64,
                1,
                0xE0,
                target,
            );

            for entry in entries.iter() {
                if entry.name[0] == 0 {
                    break;
                }

                if entry.name[0] != 0xE5 && entry.attributes & ATTR_VOLUME_ID == 0 {
                    if !(entry.name[0] == b'.' && (entry.name[1] == b' ' || entry.name[1] == b'.'))
                    {
                        if count == idx {
                            idx_entry = Some(entry.clone());
                            break;
                        }
                        count += 1;
                    }
                }
            }

            current_sector += 1;

            if let Some(cluster) = current_cluster {
                let sectors_per_cluster = self.header.sectors_per_cluster as u16;
                if current_sector % sectors_per_cluster == 0 {
                    let next_cluster = self.get_fat(cluster as usize);
                    if next_cluster >= 0x0002 && next_cluster < 0xFFF0 {
                        current_cluster = Some(next_cluster);
                        start_lba = self.cluster_to_lba(next_cluster as u32);
                        current_sector = 0;
                    }
                }
            }
        }

        idx_entry
    }

    pub fn overwrite_file(&mut self, entry: &str, data: &[u8]) -> Result<(), &'static str> {
        let cluster_size =
            self.header.sectors_per_cluster as usize * self.header.bytes_per_sector as usize;
        let data_len = data.len();
        let mut remaining_data = data_len;
        let mut data_ptr = data.as_ptr();

        let entry_p = &mut self.find_entry(entry).unwrap();

        let mut current_cluster =
            ((entry_p.first_cluster_high as u32) << 16) | (entry_p.first_cluster_low as u32);
        let mut next_cluster = self.get_fat(current_cluster as usize) as u32;
        while next_cluster >= 0x0002 && next_cluster < 0xFFF0 {
            let temp = self.get_fat(next_cluster as usize) as u32;
            self.set_fat(next_cluster as usize, 0x0000);
            next_cluster = temp;
        }

        self.set_fat(current_cluster as usize, 0xFFFF);

        while remaining_data > 0 {
            let lba = self.cluster_to_lba(current_cluster);
            let cluster_buffer = unsafe {
                (*(&raw mut crate::pmm::PADDR))
                    .malloc(cluster_size as u32)
                    .unwrap()
            };
            dma::read(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                cluster_buffer as *mut u8,
            );

            let bytes_to_write = core::cmp::min(cluster_size, remaining_data);
            unsafe {
                core::ptr::copy(data_ptr, cluster_buffer as *mut u8, bytes_to_write);
            }

            dma::write(
                OFFSET_LBA + lba,
                self.header.sectors_per_cluster,
                0xE0,
                cluster_buffer as *const u8,
            );

            remaining_data -= bytes_to_write;
            data_ptr = unsafe { data_ptr.add(bytes_to_write) };

            if remaining_data > 0 {
                let new_cluster = self
                    .get_cluster_free()
                    .ok_or("No free clusters available")?;
                self.set_fat(current_cluster as usize, new_cluster as u16);
                self.set_fat(new_cluster as usize, 0xFFF8);
                current_cluster = new_cluster;
            }

            unsafe {
                (*(&raw mut crate::pmm::PADDR)).dealloc(cluster_buffer);
            }
        }

        entry_p.size = data_len as u32;
        self.update_directory_entry(entry, entry_p);

        Ok(())
    }

    fn set_fat(&mut self, index: usize, value: u16) {

        let entries_per_sector = 512 / 2;
        let sector_offset = (index / entries_per_sector) as u64;

        let fat_lba = self.header.reserved_sectors as u64 + sector_offset;

        let entry_offset = index % entries_per_sector;

        unsafe {

            let buffer_option = (*(&raw mut crate::pmm::PADDR)).malloc(512);
            if buffer_option.is_none() {
                libk::println!("Failed to allocate memory for FAT sector");
                return;
            }

            let buffer = buffer_option.unwrap();
            let buffer_u16 = buffer as *mut u16;

            dma::read(OFFSET_LBA + fat_lba, 1, 0xE0, buffer as *mut u32);

            *buffer_u16.add(entry_offset) = value;

            dma::write(OFFSET_LBA + fat_lba, 1, 0xE0, buffer as *const u16);

            (*(&raw mut crate::pmm::PADDR)).dealloc(buffer);
        }
    }

    pub fn make_file(&self, folder: Entry, new_entry: Entry) {
        let folder_lba;
        let sectors_to_read;

        if folder.name == [32; 11] {

            folder_lba = self.header.reserved_sectors as u64
                + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64);
            sectors_to_read = ((self.header.dir_entries_count as u64 * 32)
                + (self.header.bytes_per_sector as u64 - 1))
                / self.header.bytes_per_sector as u64;
        } else {

            let folder_cluster =
                ((folder.first_cluster_high as u32) << 16) | folder.first_cluster_low as u32;
            folder_lba = self.cluster_to_lba(folder_cluster);
            sectors_to_read = self.header.sectors_per_cluster as u64;
        }

        let buffer_size = sectors_to_read as usize * self.header.bytes_per_sector as usize;
        let dir_buffer = unsafe {
            (*(&raw mut crate::pmm::PADDR))
                .malloc(buffer_size as u32)
                .unwrap()
        };

        dma::read(
            OFFSET_LBA + folder_lba,
            sectors_to_read as u8,
            0xE0,
            dir_buffer as *mut u8,
        );

        let entries = dir_buffer as *mut Entry;
        let entry_count = buffer_size / core::mem::size_of::<Entry>();
        let mut found = false;

        for i in 0..entry_count {
            let entry = unsafe { &mut *entries.add(i) };
            if entry.name[0] == 0x00 || entry.name[0] == 0xE5 {
                *entry = new_entry;
                found = true;
                break;
            }
        }

        if found {

            dma::write(
                OFFSET_LBA + folder_lba,
                sectors_to_read as u8,
                0xE0,
                dir_buffer as *const u8,
            );
        }

        unsafe {
            (*(&raw mut crate::pmm::PADDR)).dealloc(dir_buffer);
        }
    }

    fn update_directory_entry(&self, entry: &str, entry_s: &mut Entry) {
        let parent_dir = self.find_dir(entry).unwrap();

        let (lba, sectors) = if parent_dir.name == [32; 11] {
            (
                self.header.reserved_sectors as u64
                    + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64),
                ((self.header.dir_entries_count as u64 * 32) + self.header.bytes_per_sector as u64
                    - 1)
                    / self.header.bytes_per_sector as u64,
            )
        } else {
            let cluster = ((parent_dir.first_cluster_high as u32) << 16)
                | parent_dir.first_cluster_low as u32;
            (
                self.cluster_to_lba(cluster),
                self.header.sectors_per_cluster as u64,
            )
        };

        let buffer_size = sectors as usize * self.header.bytes_per_sector as usize;
        let dir_buffer = unsafe {
            (*(&raw mut crate::pmm::PADDR))
                .malloc(buffer_size as u32)
                .unwrap()
        };

        dma::read(OFFSET_LBA + lba, sectors as u8, 0xE0, dir_buffer as *mut u8);

        let entries = dir_buffer as *mut Entry;
        let entry_count = buffer_size / core::mem::size_of::<Entry>();
        let target_fat_name = self.path_to_fat_name(entry);

        for i in 0..entry_count {
            let current_entry = unsafe { &mut *entries.add(i) };
            if current_entry.name == target_fat_name {
                *current_entry = *entry_s;
                break;
            }
        }

        dma::write(OFFSET_LBA + lba, sectors as u8, 0xE0, dir_buffer as *const u8);
        unsafe {
            (*(&raw mut crate::pmm::PADDR)).dealloc(dir_buffer);
        }
    }

    pub fn cluster_to_lba(&self, cluster: u32) -> u64 {
        if cluster == 0 {
            return self.header.reserved_sectors as u64
                + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64);
        }

        let data_region_start = self.header.reserved_sectors as u64
            + (self.header.fat_count as u64 * self.header.sectors_per_fat as u64)
            + ((self.header.dir_entries_count as u64 * 32) / self.header.bytes_per_sector as u64);

        data_region_start + ((cluster as u64 - 2) * self.header.sectors_per_cluster as u64)
    }
}

pub fn split_path(path: &str) -> &[&str] {
    let part_count = path.split('/').filter(|part| !part.is_empty()).count();

    let parts_ptr = unsafe { (*(&raw mut crate::pmm::PADDR)).malloc(1024).unwrap() };

    let parts = unsafe { core::slice::from_raw_parts_mut(parts_ptr as *mut &str, part_count) };
    let mut index = 0;
    for part in path.split('/') {
        if !part.is_empty() {
            parts[index] = part;
            index += 1;
        }
    }

    parts
}
