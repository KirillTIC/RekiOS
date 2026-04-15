extern "C" {
    pub fn printk(s: *const u8);
    pub fn kmalloc(n: usize) -> *mut u8;
    pub fn kfree(ptr: *mut u8, n: usize);
    pub fn alloc_phys_frame() -> u64;
    //pub fn free_phys_frame(phys: u64); //TODO
    pub fn phys_to_virt(phys: u64) -> u64;
    pub fn mmio_r32(addr: u64) -> u32;
    pub fn mmio_w32(addr: u64, val: u32);
    pub fn pci_read32(bus: u8, dev: u8, fun: u8, off: u8) -> u32;
    pub fn pci_write32(bus: u8, dev: u8, fun: u8, off: u8, val: u32);
    pub fn pci_find_ahci(bus: *mut u8, dev: *mut u8, fun: *mut u8) -> u32;
    pub fn tsc_now_ns() -> u64;
    pub fn sleep_ms(ms: u64);
    pub fn ksym_export(name: *const u8, addr: usize);
    pub fn ksym_lookup(name: *const u8) -> usize;
}
#[macro_export]
macro_rules! kprint {
    ($s:literal) => {
        unsafe { $crate::reki::printk(concat!($s, "\0").as_ptr()) }
    };
}
