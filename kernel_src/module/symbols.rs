extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use spin::Mutex;
use core::ffi::CStr;

static SYMBOLS: Mutex<BTreeMap<String, usize>> = Mutex::new(BTreeMap::new());

pub fn export(name: &str, addr: usize) {
    SYMBOLS.lock().insert(name.into(), addr);
}
pub fn lookup(name: &str) -> Option<usize> {
    SYMBOLS.lock().get(name).copied()
}

///////// FUNCTIONS

pub extern "C" fn ksym_export(name_ptr: *const u8, addr: usize) {
    if name_ptr.is_null() { return; }
    let name = unsafe {
        CStr::from_ptr(name_ptr as *const i8)
            .to_str()
            .unwrap_or("")
    };
    export(name, addr);
}
pub extern "C" fn ksym_lookup(name_ptr: *const u8) -> usize {
    if name_ptr.is_null() { return 0; }
    let name = unsafe {
        CStr::from_ptr(name_ptr as *const i8)
            .to_str()
            .unwrap_or("")
    };
    lookup(name).unwrap_or(0)
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_printk(ptr: *const u8) {
    if ptr.is_null() { return; }
    let mut len = 0usize;
    unsafe { while *ptr.add(len) != 0 { len += 1; if len > 4096 { break; } } };
    let bytes = unsafe { core::slice::from_raw_parts(ptr, len) };
    if let Ok(s) = core::str::from_utf8(bytes) {
        crate::print!("{}", s);
    }
}
pub extern "C" fn kernel_kmalloc(n: usize) -> *mut u8 {
    if n == 0 { return core::ptr::null_mut(); }
    let mut v = alloc::vec![0u8; n];
    let ptr = v.as_mut_ptr();
    core::mem::forget(v);
    ptr
}
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_kfree(ptr: *mut u8, n: usize) {
    if ptr.is_null() || n == 0 { return; }
    unsafe { drop(alloc::vec::Vec::from_raw_parts(ptr, n, n)); }
}
pub extern "C" fn kernel_alloc_phys_frame_safe() -> u64 {
    use x86_64::structures::paging::FrameAllocator;
    let Some(fa_lock) = crate::FRAME_ALLOCATOR.get() else { return 0; };
    let mut fa = fa_lock.lock();
    let Some(frame) = fa.allocate_frame() else { return 0; };
    let phys = frame.start_address().as_u64();
    let virt = crate::phys_offset().as_u64() + phys;
    unsafe { core::ptr::write_bytes(virt as *mut u8, 0, 4096); }
    phys
}
//pub extern "C" fn kernel_free_phys_frame(phys: u64) {                   //TODO
    //use x86_64::{PhysAddr, structures::paging::PhysFrame};
    //let Some(fa_lock) = crate::FRAME_ALLOCATOR.get() else { return; };  //TODO
    //let mut fa = fa_lock.lock();
    //let frame = PhysFrame::containing_address(PhysAddr::new(phys));     //TODO    //AFTER NEW ALLOCATOR
    //fa.deallocate(frame);
//}                                                                       //TODO
pub extern "C" fn kernel_phys_to_virt(phys: u64) -> u64 {
    crate::phys_offset().as_u64() + phys
}
pub extern "C" fn kernel_mmio_r32(addr: u64) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}
pub extern "C" fn kernel_mmio_w32(addr: u64, val: u32) {
    unsafe { core::ptr::write_volatile(addr as *mut u32, val); }
}
pub extern "C" fn kernel_pci_read32(bus: u8, dev: u8, fun: u8, off: u8) -> u32 {
    crate::drivers::pci::pci_read32(bus, dev, fun, off)
}
pub extern "C" fn kernel_pci_write32(bus: u8, dev: u8, fun: u8, off: u8, val: u32) {
    crate::drivers::pci::pci_write32(bus, dev, fun, off, val);
}
//@TEMP
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn kernel_pci_find_ahci(
    out_bus: *mut u8,
    out_dev: *mut u8,
    out_fun: *mut u8
) -> u32 {
    match crate::drivers::pci::find_ahci_controller() {
        Some(d) => {
            unsafe {
                if !out_bus.is_null() { *out_bus = d.bus; }
                if !out_dev.is_null() { *out_dev = d.device; }
                if !out_fun.is_null() { *out_fun = d.function }
            }
            1
        }
        None => 0,
    }
}
pub extern "C" fn kernel_tsc_now_ns() -> u64 {
    crate::drivers::timer::tsc_now_ns()
}
pub extern "C" fn kernel_sleep_ms(ms: u64) {
    crate::drivers::timer::sleep_ms(ms);
}

// mem builtins needed by modules (core::ptr::copy_nonoverlapping etc.)
unsafe extern "C" {
    fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memset(dst: *mut u8, c: i32, n: usize) -> *mut u8;
    fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8;
    fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32;
}
pub extern "C" fn kernel_strlen(s: *const u8) -> usize {
    if s.is_null() { return 0; }
    let mut n = 0usize;
    unsafe { while *s.add(n) != 0 { n += 1; } }
    n
}

#[allow(function_casts_as_integer)]
pub fn init() {
    export("printk", kernel_printk as usize);
    export("kmalloc", kernel_kmalloc as usize);
    export("kfree", kernel_kfree as usize);
    export("alloc_phys_frame", kernel_alloc_phys_frame_safe as usize);
    //export("free_phys_frame", kernel_free_phys_frame as usize); //TODO
    export("phys_to_virt", kernel_phys_to_virt as usize);
    export("mmio_r32", kernel_mmio_r32 as usize);
    export("mmio_w32", kernel_mmio_w32 as usize);
    export("pci_read32", kernel_pci_read32 as usize);
    export("pci_write32", kernel_pci_write32  as usize);
    export("pci_find_ahci", kernel_pci_find_ahci as usize);
    export("tsc_now_ns", kernel_tsc_now_ns as usize);
    export("sleep_ms", kernel_sleep_ms as usize);
    export("ksym_export", ksym_export as usize);
    export("ksym_lookup", ksym_lookup as usize);
    export("strlen", kernel_strlen as usize);
    unsafe {
        export("memcpy",  memcpy  as *const () as usize);
        export("memset",  memset  as *const () as usize);
        export("memmove", memmove as *const () as usize);
        export("memcmp",  memcmp  as *const () as usize);
    }
    crate::println!("ksyms: exported {} symbols", SYMBOLS.lock().len());
}
