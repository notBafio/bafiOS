#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct BootInfo {
    pub mmap: MemoryMap,
    pub rsdp: Rsdp,
    pub tss: u16,
    pub vbe: VbeInfoBlock,
    pub mode: VbeModeInfoBlock,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
struct VbeInfoBlock {
    signature: [u8; 4],
    version: u16,
    oem: [u16; 2],
    dunno: [u8; 4],
    video_ptr: u32,
    memory_size: u16,
    reserved: [u8; 492],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMapEntry {
    pub base: u64,
    pub length: u64,
    pub memory_type: u32,
    pub reserved_acpi: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct MemoryMap {
    pub entries: [MemoryMapEntry; 32],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rsdp {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RsdtHeader {
    signature: [u8; 4],
    length: u32,
    revision: u8,
    checksum: u8,
    oem_id: [u8; 6],
    oem_table_id: [u8; 8],
    oem_revision: u32,
    creator_id: u32,
    creator_revision: u32,
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Rsdt {
    header: RsdtHeader,
    ptr: [u32; 10],
}

#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct GenericTable {
    signature: [u8; 4],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct VbeModeInfoBlock {
    attributes: u16,
    window_a: u8,
    window_b: u8,
    granularity: u16,
    window_size: u16,
    segment_a: u16,
    segment_b: u16,
    win_func_ptr: u32,
    pub pitch: u16,
    pub width: u16,
    pub height: u16,
    w_char: u8,
    y_char: u8,
    planes: u8,
    pub bpp: u8,
    banks: u8,
    memory_model: u8,
    bank_size: u8,
    image_pages: u8,
    reserved0: u8,
    red_mask_size: u8,
    red_field_position: u8,
    green_mask_size: u8,
    green_field_position: u8,
    blue_mask_size: u8,
    blue_field_position: u8,
    reserved_mask_size: u8,
    reserved_field_position: u8,
    direct_color_mode_info: u8,
    pub framebuffer: u32,
    reserved1: u32,
    reserved2: u16,
    lin_bytes_per_scan_line: u16,
    bnk_image_pages: u8,
    lin_image_pages: u8,
    lin_red_mask_size: u8,
    lin_red_field_position: u8,
    lin_green_mask_size: u8,
    lin_green_field_position: u8,
    lin_blue_mask_size: u8,
    lin_blue_field_position: u8,
    lin_reserved_mask_size: u8,
    lin_reserved_field_position: u8,
    max_pixel_clock: u32,
    reserved3: [u8; 189],
}

impl BootInfo {
    pub fn get_acpi(&self) {
        unsafe {
            let rsdt_ptr = self.rsdp.rsdt_address as *const Rsdt;
            let rsdt = rsdt_ptr.read();

            let entries = (rsdt.header.length as usize - core::mem::size_of::<RsdtHeader>()) / 4;

            for i in 0..entries {
                let entry = *(rsdt.ptr[i] as *const GenericTable);
                let signature = core::str::from_utf8_unchecked(&entry.signature);
            }
        }
    }

    pub fn get_mmap(&self, start: u64) -> MemoryMapEntry {
        for i in 0..32 {
            if self.mmap.entries[i].base == start {
                return self.mmap.entries[i];
            }
        }

        panic!("NOP STUPIDO COGLIONE");
    }
}

pub const BOOTINFO_NULL: BootInfo = BootInfo {
    mmap: MMAP_NULL,
    rsdp: RSDP_NULL,
    vbe: VBEINFO_NULL,
    mode: VBEBLOCK_NULL,
    tss: 0,
};

const VBEINFO_NULL: VbeInfoBlock = VbeInfoBlock {
    signature: [0; 4],
    version: 0,
    oem: [0; 2],
    dunno: [0; 4],
    video_ptr: 0,
    memory_size: 0,
    reserved: [0; 492],
};

const MMAPENTRY_NULL: MemoryMapEntry = MemoryMapEntry {
    base: 0,
    length: 0,
    memory_type: 0,
    reserved_acpi: 0,
};

const MMAP_NULL: MemoryMap = MemoryMap {
    entries: [MMAPENTRY_NULL; 32],
};

const RSDP_NULL: Rsdp = Rsdp {
    signature: [0; 8],
    checksum: 0,
    oem_id: [0; 6],
    revision: 0,
    rsdt_address: 0,
};

const VBEBLOCK_NULL: VbeModeInfoBlock = VbeModeInfoBlock {
    attributes: 0,
    window_a: 0,
    window_b: 0,
    granularity: 0,
    window_size: 0,
    segment_a: 0,
    segment_b: 0,
    win_func_ptr: 0,
    pitch: 0,
    width: 0,
    height: 0,
    w_char: 0,
    y_char: 0,
    planes: 0,
    bpp: 0,
    banks: 0,
    memory_model: 0,
    bank_size: 0,
    image_pages: 0,
    reserved0: 0,
    red_mask_size: 0,
    red_field_position: 0,
    green_mask_size: 0,
    green_field_position: 0,
    blue_mask_size: 0,
    blue_field_position: 0,
    reserved_mask_size: 0,
    reserved_field_position: 0,
    direct_color_mode_info: 0,
    framebuffer: 0,
    reserved1: 0,
    reserved2: 0,
    lin_bytes_per_scan_line: 0,
    bnk_image_pages: 0,
    lin_image_pages: 0,
    lin_red_mask_size: 0,
    lin_red_field_position: 0,
    lin_green_mask_size: 0,
    lin_green_field_position: 0,
    lin_blue_mask_size: 0,
    lin_blue_field_position: 0,
    lin_reserved_mask_size: 0,
    lin_reserved_field_position: 0,
    max_pixel_clock: 0,
    reserved3: [0; 189],
};
