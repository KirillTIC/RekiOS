extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use x86_64::{VirtAddr, structures::paging::{Mapper, Page, PageTableFlags as Flags, Size4KiB}};
use super::symbols;

fn make_exec(ptr: *const u8, len: usize) {
    let phys_off = crate::phys_offset();
    let mut pt = unsafe { crate::memory::page_table::init(phys_off) };
    let mut addr = ptr as u64 & !0xFFF;
    let end = ptr as u64 + len as u64;
    while addr < end {
        let page = Page::<Size4KiB>::containing_address(VirtAddr::new(addr));
        unsafe {
            let _ = pt.update_flags(page, Flags::PRESENT | Flags::WRITABLE)
                .map(|flush| flush.flush());
        }
        addr += 0x1000;
    }
}

#[derive(Clone, Copy, PartialEq,Debug)]
pub enum ModuleState {
    Live,
    Failed,
}
pub struct LoadedModule {
    pub name: String,
    pub state: ModuleState,
    exit_fn: unsafe extern "C" fn(),
    _code: Vec<u8>,
}

static REGISTRY: Mutex<Vec<LoadedModule>> = Mutex::new(Vec::new());

pub fn list() -> Vec<(String, ModuleState)> {
    REGISTRY.lock().iter().map(|m| (m.name.clone(), m.state)).collect()
}
pub fn unload(name: &str) -> Result<(), &'static str> {
    let reg = REGISTRY.lock();
    let pos = reg.iter().position(|m| m.name == name && m.state == ModuleState::Live)
        .ok_or("Module not found")?;
    let exit_fn = reg[pos].exit_fn;
    drop(reg);
    unsafe { exit_fn(); }
    crate::println!();
    REGISTRY.lock().retain(|m| m.name != name);
    crate::println!("km: unloaded {}", name);
    Ok(())
}

/////// LOADER

#[repr(C)] struct Ehdr { e_ident:[u8;16], e_type:u16, e_machine:u16,
    e_version:u32, e_entry:u64, e_phoff:u64, e_shoff:u64, e_flags:u32,
    e_ehsize:u16, e_phentsize:u16, e_phnum:u16, e_shentsize:u16,
    e_shnum:u16, e_shstrndx:u16 }
#[repr(C)] struct Shdr { sh_name:u32, sh_type:u32, sh_flags:u64,
    sh_addr:u64, sh_offset:u64, sh_size:u64, sh_link:u32, sh_info:u32,
    sh_addralign:u64, sh_entsize:u64 }
#[repr(C)] struct Sym  { st_name:u32, st_info:u8, st_other:u8,
    st_shndx:u16, st_value:u64, st_size:u64 }
#[repr(C)] struct Rela { r_offset:u64, r_info:u64, r_addend:i64 }
const ET_REL:u16=1; const EM_X86_64:u16=62;
const SHT_NOBITS:u32=8; const SHT_SYMTAB:u32=2; const SHT_RELA:u32=4;
const SHF_ALLOC:u64=2;
const SHN_UNDEF:u16=0; const SHN_ABS:u16=0xfff1;
const R_64:u32=1; const R_PC32:u32=2; const R_PLT32:u32=4;
const R_32S:u32=11; const R_32:u32=10; const R_GOTPCREL:u32=9;

fn sec_data<'a>(file: &'a [u8], sh: &Shdr) -> &'a [u8] {
    if sh.sh_type == SHT_NOBITS { return &[]; }
    &file[sh.sh_offset as usize .. sh.sh_offset as usize + sh.sh_size as usize]
}
fn cstr(tab: &[u8], off: usize) -> &str {
    let end = tab[off..].iter().position(|&b| b==0).unwrap_or(0);
    core::str::from_utf8(&tab[off..off+end]).unwrap_or("")
}
fn read_cstring_virt(addr: u64) -> String {
    let mut s = String::new();
    let mut p = addr as *const u8;
    unsafe { loop { let b=*p; if b==0||s.len()>64 {break;} s.push(b as char); p=p.add(1); } }
    s
}
pub fn load_km(data: &[u8]) -> Result<(String, unsafe extern "C" fn(), Vec<u8>), &'static str> {
    if data.len() < 64 { return Err("file too small"); }
    let hdr = unsafe { &*(data.as_ptr() as *const Ehdr) };
    if &hdr.e_ident[..4] != b"\x7FELF" { return Err("not an ELF"); }
    if hdr.e_ident[4] != 2             { return Err("not ELF64"); }
    if hdr.e_type    != ET_REL         { return Err("not ET_REL (.km must be a -c object)"); }
    if hdr.e_machine != EM_X86_64      { return Err("not x86-64"); }

    let sh_off = hdr.e_shoff as usize;
    let sh_n   = hdr.e_shnum as usize;
    let sh_sz  = hdr.e_shentsize as usize;
    let shdrs: Vec<&Shdr> = (0..sh_n).map(|i| unsafe {
        &*(data.as_ptr().add(sh_off + i*sh_sz) as *const Shdr)
    }).collect();

    let sym_i = shdrs.iter().position(|s| s.sh_type==SHT_SYMTAB)
        .ok_or("no .symtab")?;
    let strtab = sec_data(data, shdrs[shdrs[sym_i].sh_link as usize]);
    let symtab: Vec<&Sym> = sec_data(data, shdrs[sym_i])
        .chunks_exact(core::mem::size_of::<Sym>())
        .map(|c| unsafe { &*(c.as_ptr() as *const Sym) })
        .collect();

    let mut offs: Vec<Option<usize>> = alloc::vec![None; sh_n];
    let mut total: usize = 0;
    for (i, sh) in shdrs.iter().enumerate() {
        if sh.sh_flags & SHF_ALLOC == 0 { continue; }
        let align = (sh.sh_addralign as usize).max(1);
        total = (total + align - 1) & !(align - 1);
        offs[i] = Some(total);
        total += sh.sh_size as usize;
    }
    if total == 0 { return Err("no loadable sections"); }

    // Pre-scan for GOTPCREL: collect unique sym indices that need GOT slots
    let mut got: Vec<usize> = Vec::new(); // got[slot] = sym_idx
    for sh in &shdrs {
        if sh.sh_type != SHT_RELA { continue; }
        for chunk in sec_data(data, sh).chunks_exact(core::mem::size_of::<Rela>()) {
            let rela = unsafe { &*(chunk.as_ptr() as *const Rela) };
            if (rela.r_info & 0xFFFF_FFFF) as u32 != R_GOTPCREL { continue; }
            let si = (rela.r_info >> 32) as usize;
            if !got.contains(&si) { got.push(si); }
        }
    }
    let got_off = total;
    total += got.len() * 8; // append mini-GOT after sections

    let mut code = alloc::vec![0u8; total];
    let base = code.as_ptr() as u64;
    for (i, sh) in shdrs.iter().enumerate() {
        let Some(off) = offs[i] else { continue; };
        if sh.sh_type != SHT_NOBITS {
            let src = sec_data(data, sh);
            code[off..off+src.len()].copy_from_slice(src);
        }
    }

    let sec_virt = |i: usize| -> u64 {
        offs.get(i).and_then(|o|*o).map(|o| base+o as u64).unwrap_or(0)
    };

    // Fill mini-GOT with resolved symbol addresses
    for (slot, &si) in got.iter().enumerate() {
        let sym = symtab[si];
        let addr: u64 = if sym.st_shndx == SHN_UNDEF {
            let name = cstr(strtab, sym.st_name as usize);
            symbols::lookup(name).ok_or("unknown symbol — not exported by kernel")? as u64
        } else if sym.st_shndx == SHN_ABS {
            sym.st_value
        } else {
            sec_virt(sym.st_shndx as usize) + sym.st_value
        };
        let off = got_off + slot * 8;
        code[off..off+8].copy_from_slice(&addr.to_le_bytes());
    }

    for sh in &shdrs {
        if sh.sh_type != SHT_RELA { continue; }
        let tgt_i    = sh.sh_info as usize;
        let tgt_virt = sec_virt(tgt_i);
        let tgt_off  = offs[tgt_i].unwrap_or(0);
        for chunk in sec_data(data, sh)
            .chunks_exact(core::mem::size_of::<Rela>())
        {
            let rela   = unsafe { &*(chunk.as_ptr() as *const Rela) };
            let sym_i2 = (rela.r_info >> 32) as usize;
            let rtype  = (rela.r_info & 0xFFFF_FFFF) as u32;
            if rtype == 0 { continue; }
            let a  = rela.r_addend;
            let p  = tgt_virt + rela.r_offset;
            let po = tgt_off  + rela.r_offset as usize;

            // GOTPCREL: PC-relative reference to GOT slot (already filled above)
            if rtype == R_GOTPCREL {
                let slot = got.iter().position(|&s| s == sym_i2)
                    .ok_or("GOTPCREL sym not in GOT")?;
                let got_entry = base + got_off as u64 + slot as u64 * 8;
                let d = got_entry as i64 + a - p as i64;
                let v = i32::try_from(d).map_err(|_| "GOTPCREL overflow")?;
                code[po..po+4].copy_from_slice(&v.to_le_bytes());
                continue;
            }

            let sym = symtab[sym_i2];
            let s: u64 = if sym.st_shndx == SHN_UNDEF {
                let name = cstr(strtab, sym.st_name as usize);
                symbols::lookup(name)
                    .ok_or("unknown symbol — not exported by kernel")? as u64
            } else if sym.st_shndx == SHN_ABS {
                sym.st_value
            } else {
                sec_virt(sym.st_shndx as usize) + sym.st_value
            };
            match rtype {
                R_64  => {
                    let v = (s as i64 + a) as u64;
                    code[po..po+8].copy_from_slice(&v.to_le_bytes());
                }
                R_PC32|R_PLT32 => {
                    let d = s as i64 + a - p as i64;
                    let v = i32::try_from(d)
                        .map_err(|_| "PC32 overflow: module too far from kernel")?;
                    code[po..po+4].copy_from_slice(&v.to_le_bytes());
                }
                R_32S => {
                    let v = i32::try_from(s as i64+a)
                        .map_err(|_|"R_32S overflow")?;
                    code[po..po+4].copy_from_slice(&v.to_le_bytes());
                }
                R_32 => {
                    let v = u32::try_from((s as i64+a) as u64)
                        .map_err(|_|"R_32 overflow")?;
                    code[po..po+4].copy_from_slice(&v.to_le_bytes());
                }
                t => {
                    crate::println!("km: unknown relocation {}", t);
                    return Err("unknown relocation");
                }
            }
        }
    }

    let mut init_addr: Option<u64> = None;
    let mut exit_addr: Option<u64> = None;
    let mut name = String::from("unknown");
    for sym in &symtab {
        if sym.st_shndx == SHN_UNDEF || sym.st_shndx == SHN_ABS { continue; }
        let sname = cstr(strtab, sym.st_name as usize);
        let addr  = sec_virt(sym.st_shndx as usize) + sym.st_value;
        match sname {
            "module_init" => init_addr = Some(addr),
            "module_exit" => exit_addr = Some(addr),
            "module_name" => name      = read_cstring_virt(addr),
            _ => {}
        }
    }
    let init_addr = init_addr.ok_or("no module_init symbol")?;
    let exit_addr = exit_addr.ok_or("no module_exit symbol")?;

    make_exec(code.as_ptr(), code.len());

    let init_fn: extern "C" fn() -> i32 = unsafe { core::mem::transmute(init_addr) };
    let init_ret = init_fn();
    crate::println!();
    if init_ret != 0 {
        return Err("module_init() returned non-zero");
    }
    let exit_fn: unsafe extern "C" fn() = unsafe { core::mem::transmute(exit_addr) };

    crate::println!("km: '{}' loaded ({} bytes of code)", name, total);
    Ok((name, exit_fn, code))
}

pub fn insmod(data: &[u8]) -> Result<(), &'static str> {
    let (name, exit_fn, code) = load_km(data)?;
    REGISTRY.lock().push(LoadedModule {
        name,
        state: ModuleState::Live,
        exit_fn,
        _code: code,
    });
    Ok(())
}
