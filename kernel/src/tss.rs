#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
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
