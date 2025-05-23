#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct TaskStateSegment {
    pub link: u16,
    pub padding_0: u16,
    pub esp0: u32,
    pub ss0: u16,
    pub padding_1: u16,
    pub esp1: u32,
    pub ss1: u16,
    pub padding_2: u16,
    pub esp2: u32,
    pub ss2: u16,
    pub padding_3: u16,
    pub cr3: u32,
    pub eip: u32,
    pub eflags: u32,
    pub eax: u32,
    pub ecx: u32,
    pub edx: u32,
    pub ebx: u32,
    pub esp: u32,
    pub ebp: u32,
    pub esi: u32,
    pub edi: u32,
    pub es: u16,
    pub padding_4: u16,
    pub cs: u16,
    pub padding_5: u16,
    pub ss: u16,
    pub padding_6: u16,
    pub ds: u16,
    pub padding_7: u16,
    pub fs: u16,
    pub padding_8: u16,
    pub gs: u16,
    pub padding_9: u16,
    pub ldtr: u16,
    pub padding_10: u16,
    pub padding_11: u16,
    pub iopb: u16,
    pub ssp: u32,
}

pub static mut TSS: TaskStateSegment = TaskStateSegment {
    link: 0,
    padding_0: 0,
    esp0: 0x30_0000,
    ss0: 0x10,
    padding_1: 0,
    esp1: 0,
    ss1: 0,
    padding_2: 0,
    esp2: 0,
    ss2: 0,
    padding_3: 0,
    cr3: 0,
    eip: 0,
    eflags: 0,
    eax: 0,
    ecx: 0,
    edx: 0,
    ebx: 0,
    esp: 0,
    ebp: 0,
    esi: 0,
    edi: 0,
    es: 0,
    padding_4: 0,
    cs: 0,
    padding_5: 0,
    ss: 0,
    padding_6: 0,
    ds: 0,
    padding_7: 0,
    fs: 0,
    padding_8: 0,
    gs: 0,
    padding_9: 0,
    ldtr: 0,
    padding_10: 0,
    padding_11: 0,
    iopb: 0,
    ssp: 0,
};
