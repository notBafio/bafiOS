#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct TargaHdr {
    pub magic1: u8,
    pub colormap: u8,
    encoding: u8,
    cmaporig: u16,
    cmaplen: u16,
    cmapdepth: u8,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
    pub bpp: u8,
    pub pixeltype: u8,
}
