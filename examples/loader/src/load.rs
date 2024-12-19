use core::panic;
use core::{
    slice::{from_raw_parts, from_raw_parts_mut},
    cmp::min,
};

use axlog::debug;

use elf::{
    abi::{ET_DYN, ET_EXEC},
    endian::LittleEndian,
    ElfBytes,
};

const PLASH_START: usize = 0xffff_ffc0_2200_0000;
const MAX_APP_SIZE: usize = 0x100000;
const EXEC_ZONE_START: usize = 0xffff_ffc0_8010_0000;

pub fn load_elf() -> u64 {
    debug!("Load payload ...");
    let elf_size = unsafe { *(PLASH_START as *const usize) };
    debug!("ELF size: 0x{:x}", elf_size);
    let elf_slice = unsafe { from_raw_parts((PLASH_START + 0x8) as *const u8, elf_size) };
    let elf: ElfBytes<'_, LittleEndian> =
        ElfBytes::<LittleEndian>::minimal_parse(elf_slice).expect("Failed to parse ELF");
    let elf_hdr = elf.ehdr;

    let run_code =
        unsafe { from_raw_parts_mut(EXEC_ZONE_START as *mut u8, MAX_APP_SIZE) };

    // debug_elf(&elf, elf_slice);

    let entry: u64;
    if elf_hdr.e_type == ET_EXEC {
        // static and position independent executable
        load_exec(&elf, elf_slice, run_code);
        entry = elf_hdr.e_entry;
    } else if elf_hdr.e_type == ET_DYN {
        load_dyn(&elf, elf_slice, run_code);
        entry = EXEC_ZONE_START as u64 + elf_hdr.e_entry;
    } else {
        panic!("Invalid ELF type");
    }
    debug!("Entry: 0x{:x}", entry);
    return entry;
}

fn load_exec(elf: &ElfBytes<LittleEndian>, elf_slice: &[u8], run_code: &mut [u8]) {
    let text_shdr = elf
        .section_header_by_name(".text")
        .expect("section table should be parseable")
        .expect("elf should have a .text section");
    let text_slice = elf_slice
        .get(text_shdr.sh_offset as usize..)
        .expect("text section should be in bounds");
    let copy_size = min(run_code.len(), text_slice.len());
    run_code[..copy_size].copy_from_slice(&text_slice[..copy_size]);
}

fn load_dyn(elf: &ElfBytes<LittleEndian>, elf_slice: &[u8], run_code: &mut [u8]) {
    let phdrs = elf.segments().expect("Failed to parse program headers");
    for phdr in phdrs {
        if phdr.p_type != elf::abi::PT_LOAD {
            continue;
        }
        // debug!("phdr offset: 0x{:x}", phdr.p_offset);
        // debug!("phdr vaddr: 0x{:x}", phdr.p_vaddr);
        // debug!("phdr paddr: 0x{:x}", phdr.p_paddr);
        // debug!("phdr filesz: 0x{:x}", phdr.p_filesz);
        // debug!("phdr memsz: 0x{:x}\n", phdr.p_memsz);
        load_segment(run_code, elf_slice, phdr.p_vaddr as usize, phdr.p_offset as usize, phdr.p_filesz as usize, phdr.p_memsz as usize);
    }
    modify_plt(elf);
}

fn load_segment(run_code: &mut [u8], elf_slice: &[u8], p_vaddr: usize, p_offset: usize, p_filesz: usize, p_memsz: usize) {
    // copy the segment into the executable zone
    // if memz is larger than filesz, zero out the rest
    let run_code_offset = p_vaddr;
    run_code[run_code_offset..run_code_offset + p_filesz]
        .copy_from_slice(&elf_slice[p_offset..p_offset + p_filesz]);
    if p_memsz > p_filesz {
        let zero_size = min(
            run_code.len() - p_filesz,
            p_memsz - p_filesz,
        );
        run_code[run_code_offset + p_filesz..run_code_offset + p_filesz + zero_size].fill(0);
    }
}

fn modify_plt(elf: &ElfBytes<LittleEndian>) {
    let (dynsym_table, dynstr_table) = elf.dynamic_symbol_table()
        .expect("Failed to parse dynamic symbol table")
        .expect("ELF should have a dynamic symbol table");
    let rela_shdr = elf
        .section_header_by_name(".rela.plt")
        .expect("section table should be parseable")
        .expect("elf should have a .rela.plt section");

    let relas = elf.section_data_as_relas(&rela_shdr)
        .expect("Failed to parse .rela.dyn section");

    for rela in relas {
        // get the r_sym'th symbol from the dynamic symbol table
        let sym = dynsym_table.get(rela.r_sym as usize).expect("Failed to get symbol");
        let rela_name = dynstr_table.get(sym.st_name as usize).expect("Failed to get symbol name");
        // debug!("Rela sym: {}", rela_name);
        let func = super::abi::AbiFunction::from_name(rela_name).expect("Failed to find abi function");
        unsafe {
            *((EXEC_ZONE_START as u64 + rela.r_offset) as *mut usize) = func.addr();
            debug!("{} at : 0x{:x}", rela_name,*((EXEC_ZONE_START as u64 + rela.r_offset) as *const usize));
        }
    }

}

#[allow(dead_code)]
fn debug_elf(_elf: &ElfBytes<LittleEndian>, _elf_slice: &[u8]) {
    // let got_shdr = elf
    //     .section_header_by_name(".got")
    //     .expect("section table should be parseable")
    //     .expect("elf should have a .got section");

    // let plt_shdr = elf
    //     .section_header_by_name(".plt")
    //     .expect("section table should be parseable")
    //     .expect("elf should have a .plt section");

    // debug!("GOT: 0x{:x} - 0x{:x}", got_shdr.sh_offset, got_shdr.sh_offset + got_shdr.sh_size);
    // debug!("PLT: 0x{:x} - 0x{:x}", plt_shdr.sh_offset, plt_shdr.sh_offset + plt_shdr.sh_size);

    // let interp_shdr = elf
    //     .section_header_by_name(".interp")
    //     .expect("section table should be parseable")
    //     .expect("elf should have a .interp section");

    // unsafe {
    //     let interp = from_raw_parts(
    //         elf_slice.as_ptr().add(interp_shdr.sh_offset as usize),
    //         interp_shdr.sh_size as usize,
    //     );
    //     debug!("interp: {:?}", core::str::from_utf8_unchecked(interp));
    // }

    // let (dynsym_table, dynstr_table) = elf.dynamic_symbol_table()
    //     .expect("Failed to parse dynamic symbol table")
    //     .expect("ELF should have a dynamic symbol table");

    // for dynsym in dynsym_table.iter() {
    //     info!("dynsym st_name: {}", dynsym.st_name);
    //     debug!("dynsym: {}, type: {}", dynstr_table.get(dynsym.st_name as usize).unwrap(), elf::to_str::st_symtype_to_str(dynsym.st_symtype()).unwrap());
    // }

    // let dynamic_table = elf.dynamic()
    //     .expect("Failed to parse dynamic section")
    //     .expect("ELF should have a dynamic section");

    // info!("\n");
    // for dyn_entry in dynamic_table {
    //     info!("dyn_entry tag: {}", elf::to_str::d_tag_to_str(dyn_entry.d_tag).unwrap());
    //     debug!("dyn_entry val: 0x{:x}", dyn_entry.d_val());
    // }

    // let (symtab, strtab) = elf.symbol_table()
    //     .expect("Failed to parse symbol table")
    //     .expect("ELF should have a symbol table");

    // for sym in symtab {
    //     info!("sym st_name: {}", sym.st_name);
    //     debug!("sym: {}, type: {}", strtab.get(sym.st_name as usize).unwrap(), elf::to_str::st_symtype_to_str(sym.st_symtype()).unwrap());
    // }

    // let rela_shdr = elf
    //     .section_header_by_name(".rela.plt")
    //     .expect("section table should be parseable")
    //     .expect("elf should have a .rela.dyn section");

    // let relas = elf.section_data_as_relas(&rela_shdr)
    //     .expect("Failed to parse .rela.dyn section");

    // let args = ['\n' as usize, 0, 0, 0];
    // for rela in relas {
    //     info!("Rela r_sym: {}", rela.r_sym);
    //     // get the r_sym'th symbol from the dynamic symbol table
    //     let sym = dynsym_table.get(rela.r_sym as usize).unwrap();
    //     let rela_name = dynstr_table.get(sym.st_name as usize).unwrap();
    //     info!("Rela sym: {}", rela_name);

    //     for (name, func) in STR_TO_FUNC.iter() {
    //         if rela_name == *name {
    //             debug!("Rela func: {:?}", func);
    //             func.call(args);
    //         }
    //     }
    // }

    // info!("\n");
    // let phdrs = elf.segments().expect("Failed to parse program headers");
    // for phdr in phdrs {
    //     if phdr.p_type != elf::abi::PT_LOAD {
    //         continue;
    //     }
    //     debug!(
    //         "\nphdr type: {}",
    //         elf::to_str::p_type_to_str(phdr.p_type).unwrap()
    //     );
    //     debug!("phdr offset: 0x{:x}", phdr.p_offset);
    //     debug!("phdr vaddr: 0x{:x}", phdr.p_vaddr);
    //     debug!("phdr paddr: 0x{:x}", phdr.p_paddr);
    //     debug!("phdr filesz: 0x{:x}", phdr.p_filesz);
    //     debug!("phdr memsz: 0x{:x}\n", phdr.p_memsz);
    // }
}
