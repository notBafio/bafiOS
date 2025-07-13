use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub base: u32,
    pub size: u32,
}

pub struct PMM {
    pub sections: Vec<Section>,
    pub ram_size: u32,
}

const INITIAL_CAPACITY: usize = 2048;

pub static mut PADDR: PMM = PMM {
    sections: Vec::new(),
    ram_size: 0,
};

impl PMM {
    pub fn init(&mut self) {
        let info = unsafe { crate::BOOTINFO };

        self.ram_size = info.get_mmap(0x10_0000 as u64).length as u32;

        self.sections.push(Section {
            base: 0xA0_0000,
            size: 0,
        });
    }

    pub fn malloc(&mut self, size: u32) -> Option<u32> {
        self.sections.sort_by_key(|s| s.base);
        let mut candidate = 0xA0_0000;

        for section in &self.sections {
            let candidate_aligned = self.align_up(candidate);

            if candidate_aligned + size <= section.base {
                if candidate_aligned + size <= self.ram_size {
                    self.sections.push(Section {
                        base: candidate_aligned,
                        size,
                    });

                    if self.check_overlaps() {
                        if let Ok(pos) = self
                            .sections
                            .binary_search_by_key(&candidate_aligned, |s| s.base)
                        {
                            self.sections.remove(pos);
                            return None;
                        }
                    }
                    return Some(candidate_aligned);
                } else {
                    return None;
                }
            }

            let section_end = self.align_up(section.base + section.size);
            if section_end > candidate {
                candidate = section_end;
            }
        }

        let candidate_aligned = self.align_up(candidate);
        if candidate_aligned + size <= self.ram_size {
            self.sections.push(Section {
                base: candidate_aligned,
                size,
            });

            if self.check_overlaps() {
                if let Ok(pos) = self
                    .sections
                    .binary_search_by_key(&candidate_aligned, |s| s.base)
                {
                    self.sections.remove(pos);
                    return None;
                }
            }
            return Some(candidate_aligned);
        }
        None
    }

    pub fn check_overlaps(&self) -> bool {
        false
    }

    pub fn dealloc(&mut self, base: u32) {
        if base == 0 {
            return;
        }

        self.sections.sort_by_key(|s| s.base);
        if let Ok(pos) = self.sections.binary_search_by_key(&base, |s| s.base) {
            self.sections.remove(pos);
        }
    }

    pub fn add_fb(&mut self, base: u32, size: u32) {
        self.sections.push(Section { base, size });
    }

    pub fn align_up(&self, address: u32) -> u32 {
        (address + 0xFFF) & !0xFFF
    }
}
