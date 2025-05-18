#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf32Ehdr {
    e_ident: [u8; 16],
    e_type: u16,
    e_machine: u16,
    e_version: u32,
    e_entry: u32,
    e_phoff: u32,
    e_shoff: u32,
    e_flags: u32,
    e_ehsize: u16,
    e_phentsize: u16,
    e_phnum: u16,
    e_shentsize: u16,
    e_shnum: u16,
    e_shstrndx: u16,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Elf32Phdr {
    ph_type: u32,
    ph_offset: u32,
    ph_vaddr: u32,
    ph_paddr: u32,
    ph_filesz: u32,
    ph_memsz: u32,
    ph_flags: u32,
    ph_align: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Elf32Shdr {
    sh_name: u32,
    sh_type: u32,
    sh_flags: u32,
    sh_addr: u32,
    sh_offset: u32,
    sh_size: u32,
    sh_link: u32,
    sh_info: u32,
    sh_align: u32,
    sh_entsize: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Elf32Sym {
    st_name: u32,
    st_value: u32,
    st_size: u32,
    st_info: u8,
    st_other: u8,
    st_shndx: u16,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Elf32Rel {
    r_offset: u32,
    r_info: u32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Elf32Rela {
    r_offset: u32,
    r_info: u32,
    r_addend: i32,
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
struct Elf32Dyn {
    d_tag: i32,
    d_val: u32,
}
const R_386_NONE: u32 = 0;
const R_386_32: u32 = 1;
const R_386_PC32: u32 = 2;
const R_386_GOT32: u32 = 3;
const R_386_PLT32: u32 = 4;
const R_386_COPY: u32 = 5;
const R_386_GLOB_DAT: u32 = 6;
const R_386_JMP_SLOT: u32 = 7;
const R_386_RELATIVE: u32 = 8;
const R_386_GOTOFF: u32 = 9;
const R_386_GOTPC: u32 = 10;
const R_386_32PLT: u32 = 11;
const PT_DYNAMIC: u32 = 2;
const PT_LOAD: u32 = 1;
const DT_NULL: i32 = 0;
const DT_NEEDED: i32 = 1;
const DT_REL: i32 = 17;
const DT_RELSZ: i32 = 18;
const DT_RELA: i32 = 7;
const DT_RELASZ: i32 = 8;
const DT_STRTAB: i32 = 5;
const DT_STRSZ: i32 = 10;
const DT_SYMTAB: i32 = 6;
const DT_SYMENT: i32 = 11;
const DT_PLTGOT: i32 = 3;
const DT_PLTRELSZ: i32 = 2;
const DT_PLTREL: i32 = 20;
const DT_JMPREL: i32 = 23;

#[derive(Debug, Copy, Clone)]
pub struct Elf32Load {
    file_hdr: Elf32Ehdr,
    base: u32,

    strtab: u32,
    symtab: u32,
    syment: u32,

    rel: u32,
    relent: u32,
    relsz: u32,

    rela: u32,
    relaent: u32,
    relasz: u32,

    jmprel: u32,
    pltrel: u32,
    pltrelsz: u32,
    pltgot: u32,

    init: u32,
    init_array: u32,
    init_arraysz: u32,

    inited: bool,
    relocated: bool,

    strlen: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct CrossReloc {
    name: *const u8,
    got_ptr: u32,
}

pub struct RelTable {
    base_ptr: u32,
    len: u32,
}

pub static mut REL_TABLE: RelTable = RelTable {
    base_ptr: 0,
    len: 0,
};

pub fn load_lib(fname: &str, _args: Option<&[u32]>) -> Result<u32, &'static str> {
    let file = crate::io::File::new(fname);

    if file.size < core::mem::size_of::<Elf32Ehdr>() as u32 {
        return Err("File too small to be an ELF");
    }

    let hdr_ptr = file.ptr as *const Elf32Ehdr;
    if (hdr_ptr as usize) % core::mem::align_of::<Elf32Ehdr>() != 0 {
        return Err("Unaligned ELF header");
    }

    let hdr = unsafe { *hdr_ptr };

    if !check_header(&hdr) {
        return Err("Invalid ELF header");
    }

    let base_addr = match crate::syscall::malloc(file.size) {
        0 => return Err("Failed to allocate memory for ELF"),
        addr => addr,
    };

    if base_addr % 4096 != 0 {
        return Err("Base address not page-aligned");
    }

    let mut loader = Elf32Load {
        file_hdr: hdr,
        base: base_addr,

        strtab: 0,
        strlen: 0,

        symtab: 0,
        syment: 0,

        rel: 0,
        relent: 8,
        relsz: 0,

        rela: 0,
        relaent: 12,
        relasz: 0,

        jmprel: 0,
        pltrel: 0,
        pltrelsz: 0,
        pltgot: 0,

        init: 0,
        init_array: 0,
        init_arraysz: 0,

        inited: false,
        relocated: false,
    };

    if hdr.e_phoff == 0 || hdr.e_phnum == 0 {
        return Err("No program headers found");
    }

    let phdr_size = hdr.e_phnum as u32 * hdr.e_phentsize as u32;
    if hdr.e_phoff + phdr_size > file.size {
        return Err("Program headers outside file bounds");
    }

    let prghdr = unsafe {
        core::slice::from_raw_parts(
            (file.ptr + hdr.e_phoff) as *const Elf32Phdr,
            hdr.e_phnum as usize,
        )
    };

    for header in prghdr {
        match header.ph_type {
            PT_LOAD => {
                if header.ph_offset + header.ph_filesz > file.size {
                    return Err("Load segment outside file bounds");
                }

                if header.ph_vaddr.checked_add(header.ph_filesz).is_none() {
                    return Err("Load address arithmetic overflow");
                }

                if (loader.base + header.ph_vaddr) % 4 != 0 {
                    return Err("Load address not properly aligned");
                }

                let src_base = file.ptr + header.ph_offset;
                let dst_base = loader.base + header.ph_vaddr;
                let size = header.ph_filesz as usize;

                unsafe {
                    for i in 0..size {
                        let byte = *((src_base + i as u32) as *const u8);
                        *((dst_base + i as u32) as *mut u8) = byte;
                    }
                }
                if header.ph_memsz > header.ph_filesz {
                    let bss_start = loader.base + header.ph_vaddr + header.ph_filesz;
                    let bss_size = header.ph_memsz - header.ph_filesz;

                    if header.ph_filesz.checked_add(bss_size).is_none() {
                        return Err("BSS size arithmetic overflow");
                    }

                    unsafe {
                        for i in 0..bss_size as usize {
                            *((bss_start + i as u32) as *mut u8) = 0;
                        }
                    }
                }
            }

            PT_DYNAMIC => {
                if header.ph_offset + header.ph_filesz > file.size {
                    return Err("Dynamic segment outside file bounds");
                }

                let dyn_ptr = (file.ptr + header.ph_offset) as *const Elf32Dyn;
                let num_dyn_entries = header.ph_filesz as usize / core::mem::size_of::<Elf32Dyn>();
                let dynamic_section =
                    unsafe { core::slice::from_raw_parts(dyn_ptr, num_dyn_entries) };

                for dyn_entry in dynamic_section {
                    match dyn_entry.d_tag {
                        DT_NULL => break,

                        DT_STRTAB => loader.strtab = dyn_entry.d_val,
                        DT_STRSZ => loader.strlen = dyn_entry.d_val,
                        DT_SYMTAB => loader.symtab = dyn_entry.d_val,
                        DT_SYMENT => loader.syment = dyn_entry.d_val,

                        DT_REL => loader.rel = dyn_entry.d_val,
                        DT_RELSZ => loader.relsz = dyn_entry.d_val,

                        DT_RELA => loader.rela = dyn_entry.d_val,
                        DT_RELASZ => loader.relasz = dyn_entry.d_val,

                        DT_PLTGOT => loader.pltgot = dyn_entry.d_val,
                        DT_PLTRELSZ => loader.pltrelsz = dyn_entry.d_val,
                        DT_PLTREL => loader.pltrel = dyn_entry.d_val,
                        DT_JMPREL => loader.jmprel = dyn_entry.d_val,

                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    if loader.strtab != 0 && loader.strlen != 0 {
        handle_strtab(file.ptr, &loader)?;
    }

    if loader.jmprel != 0 && loader.pltrelsz != 0 {
        handle_pltrel(&loader)?;
    }

    if loader.rel != 0 && loader.relsz != 0 {
        handle_rel(&loader)?;
    }

    Ok(loader.base + hdr.e_entry)
}

fn handle_rel(elf: &Elf32Load) -> Result<(), &'static str> {
    if elf.rel == 0 || elf.relsz == 0 {
        return Ok(());
    }

    let rel_count = elf.relsz / elf.relent;
    if rel_count == 0 {
        return Ok(());
    }

    let rels = unsafe {
        core::slice::from_raw_parts((elf.base + elf.rel) as *const Elf32Rel, rel_count as usize)
    };

    for rel in rels {
        let rel_type = rel.r_info & 0xFF;
        let sym_index = rel.r_info >> 8;

        match rel_type {
            R_386_32 => {
                let sym = get_symbol(elf, sym_index)?;
                let relocation_addr = (elf.base + rel.r_offset) as *mut u32;

                unsafe {
                    let current_val = *relocation_addr;
                    *relocation_addr = current_val + elf.base + sym.st_value;
                }
            }

            R_386_PC32 => {
                let sym = get_symbol(elf, sym_index)?;
                let relocation_addr = (elf.base + rel.r_offset) as *mut u32;

                unsafe {
                    let current_val = *relocation_addr;
                    *relocation_addr =
                        current_val + (elf.base + sym.st_value) - (elf.base + rel.r_offset);
                }
            }

            R_386_RELATIVE => {
                let relocation_addr = (elf.base + rel.r_offset) as *mut u32;

                unsafe {
                    let current_val = *relocation_addr;
                    *relocation_addr = current_val + elf.base;
                }
            }

            R_386_NONE => {}
            _ => {}
        }
    }

    Ok(())
}

fn handle_pltrel(elf: &Elf32Load) -> Result<(), &'static str> {
    /*if elf.jmprel == 0 || elf.pltrelsz == 0 {
        return Ok(());
    }

    let rel_count = elf.pltrelsz / elf.relent;
    if rel_count == 0 {
        return Ok(());
    }

    let rels = unsafe {
        core::slice::from_raw_parts(
            (elf.base + elf.jmprel) as *const Elf32Rel,
            rel_count as usize
        )
    };

    for rel in rels {
        let rel_type = rel.r_info & 0xFF;
        let sym_index = rel.r_info >> 8;

        match rel_type {
            R_386_JMP_SLOT | R_386_GLOB_DAT => {
                let sym = get_symbol(elf, sym_index)?;
                let sym1_name = (elf.base + elf.strtab + sym.st_name) as *const u8;
                let external_rels = unsafe { REL_TABLE.len as usize };
                let mut got_entry = (elf.base + rel.r_offset) as *mut u32;

                if sym.st_shndx == 0 {

                    unsafe {

                        let external_relocs = core::slice::from_raw_parts(
                            REL_TABLE.base_ptr as *const CrossReloc,
                            external_rels,
                        );

                        for i in 0..external_relocs.len() {
                            let entry = external_relocs[i as usize];

                            if strcmp(sym1_name, entry.name) == 0 {
                                got_entry = entry.got_ptr as *mut u32;
                            }
                        }

                        if got_entry as usize != 0 && got_entry as u32 % 4 == 0{
                            *got_entry = elf.base + sym.st_value;
                        }
                    }

                } else {
                    let got_entry = elf.base + rel.r_offset;
                    let name_ptr =  elf.base + elf.strtab + sym.st_name;

                    let new_reloc = CrossReloc {
                        name: name_ptr as *const u8,
                        got_ptr: got_entry,
                    };

                    unsafe {
                        *(REL_TABLE.base_ptr as *mut CrossReloc) = new_reloc;
                        REL_TABLE.len += 1
                    }
                }
            },

            _ => {}
        }
    }*/

    Ok(())
}

fn get_symbol(elf: &Elf32Load, index: u32) -> Result<Elf32Sym, &'static str> {
    if elf.symtab == 0 || elf.syment == 0 {
        return Err("");
    }

    if index > 10000 {
        return Err("");
    }

    let sym_addr = elf.base + elf.symtab + (index * elf.syment);

    if sym_addr % 4 != 0 {
        return Err("");
    }

    let sym_ptr = sym_addr as *const Elf32Sym;

    let symbol = unsafe { *sym_ptr };

    Ok(symbol)
}

fn handle_strtab(file_ptr: u32, elf: &Elf32Load) -> Result<(), &'static str> {
    if elf.strtab == 0 || elf.strlen == 0 {
        return Ok(());
    }

    if file_ptr == 0 || elf.base == 0 {
        return Err("");
    }

    let src_ptr = (file_ptr + elf.strtab) as *const u8;
    let dst_ptr = (elf.base + elf.strtab) as *mut u8;

    let len = elf.strlen as usize;
    unsafe {
        for i in 0..len {
            *dst_ptr.add(i) = *src_ptr.add(i);
        }
    }

    Ok(())
}

fn check_header(header: &Elf32Ehdr) -> bool {
    if header.e_ident[0] != 0x7f || &header.e_ident[1..4] != b"ELF" {
        return false;
    }
    if header.e_ident[4] != 1 {
        return false;
    }
    if header.e_ident[5] != 1 {
        return false;
    }

    true
}

pub fn load_elf(filename: &str, args: Option<&[u32]>) -> Result<(), &'static str> {
    unsafe {
        REL_TABLE.base_ptr = crate::syscall::malloc(10000);
    }

    let entry_point = load_lib(filename, args)?;

    unsafe {
        crate::syscall::free(REL_TABLE.base_ptr);
    }

    if entry_point != 0 {
        crate::syscall::add_task(entry_point, args);
        Ok(())
    } else {
        Err("")
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn strcmp(s1: *const u8, s2: *const u8) -> i32 {
    let mut i = 0;
    loop {
        let a = unsafe { *s1.add(i) };
        let b = unsafe { *s2.add(i) };

        if a != b {
            return (a as i32) - (b as i32);
        }
        if a == 0 {
            break;
        }

        i += 1;
    }

    0
}
