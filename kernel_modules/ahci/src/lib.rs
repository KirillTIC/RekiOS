#![no_std]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

mod reki { include!("../../reki.rs"); }
use reki::*;

const HBA_CAP:  u64 = 0x00;
const HBA_GHC:  u64 = 0x04;
const HBA_IS:   u64 = 0x08;
const HBA_PI:   u64 = 0x0C;
const HBA_CAP2: u64 = 0x24;
const HBA_BOHC: u64 = 0x28;

const P_CLB:  u64 = 0x00;
const P_CLBU: u64 = 0x04;
const P_FB:   u64 = 0x08;
const P_FBU:  u64 = 0x0C;
const P_IS:   u64 = 0x10;
const P_CMD:  u64 = 0x18;
const P_TFD:  u64 = 0x20;
const P_SSTS: u64 = 0x28;
const P_SERR: u64 = 0x30;
const P_CI:   u64 = 0x38;

struct AhciState {
    abar:      u64,
    port_base: u64,
    clb:       u64,
    ct:        u64,
}

static mut AHCI: Option<AhciState> = None;

#[inline] fn hr32(abar: u64, off: u64) -> u32 { unsafe { mmio_r32(abar + off) } }
#[inline] fn hw32(abar: u64, off: u64, v: u32) { unsafe { mmio_w32(abar + off, v) } }
#[inline] fn pr32(pb: u64, off: u64) -> u32  { unsafe { mmio_r32(pb + off) } }
#[inline] fn pw32(pb: u64, off: u64, v: u32) { unsafe { mmio_w32(pb + off, v) } }

unsafe fn wmem32(virt: u64, off: usize, v: u32) {
    core::ptr::write_volatile((virt + off as u64) as *mut u32, v);
}
unsafe fn rmem32(virt: u64, off: usize) -> u32 {
    core::ptr::read_volatile((virt + off as u64) as *const u32)
}
unsafe fn wmem8(virt: u64, off: usize, v: u8) {
    core::ptr::write_volatile((virt + off as u64) as *mut u8, v);
}
unsafe fn zero_frame(phys: u64) {
    let virt = phys_to_virt(phys) as *mut u8;
    core::ptr::write_bytes(virt, 0, 4096);
}

fn poll_clear(abar: u64, off: u64, mask: u32, timeout_ms: u64) -> bool {
    let deadline = unsafe { tsc_now_ns() } + timeout_ms * 1_000_000;
    loop {
        if hr32(abar, off) & mask == 0 { return true; }
        if unsafe { tsc_now_ns() } > deadline { return false; }
        unsafe { sleep_ms(1) };
    }
}
fn port_poll_clear(pb: u64, off: u64, mask: u32, timeout_ms: u64) -> bool {
    let deadline = unsafe { tsc_now_ns() } + timeout_ms * 1_000_000;
    loop {
        if pr32(pb, off) & mask == 0 { return true; }
        if unsafe { tsc_now_ns() } > deadline { return false; }
        unsafe { sleep_ms(1) };
    }
}

unsafe fn ahci_init() -> bool {
    let mut bus: u8 = 0; let mut dev: u8 = 0; let mut fun: u8 = 0;
    if pci_find_ahci(&mut bus, &mut dev, &mut fun) == 0 {
        printk(b"ahci: controller not found\0".as_ptr());
        return false;
    }

    let cmd = pci_read32(bus, dev, fun, 0x04);
    pci_write32(bus, dev, fun, 0x04, cmd | (1 << 1) | (1 << 2));

    let bar5_lo = pci_read32(bus, dev, fun, 0x24);
    let bar5_type = (bar5_lo >> 1) & 0x3;
    let mut bar5_phys = (bar5_lo & !0xF) as u64;
    if bar5_type == 0x2 {
        let bar6 = pci_read32(bus, dev, fun, 0x28) as u64;
        bar5_phys |= bar6 << 32;
    }
    let abar = phys_to_virt(bar5_phys);

    let cap2 = hr32(abar, HBA_CAP2);
    if cap2 & 1 != 0 {
        let bohc = hr32(abar, HBA_BOHC);
        hw32(abar, HBA_BOHC, bohc | (1 << 1));
        let deadline = tsc_now_ns() + 2_000_000_000;
        loop {
            if hr32(abar, HBA_BOHC) & (1 << 4) == 0 { break; }
            if tsc_now_ns() > deadline { break; }
            sleep_ms(10);
        }
    }

    let ghc = hr32(abar, HBA_GHC);
    hw32(abar, HBA_GHC, ghc | 0x8000_0001);
    if !poll_clear(abar, HBA_GHC, 0x1, 1000) {
        printk(b"ahci: HBA reset timeout\0".as_ptr());
        return false;
    }
    sleep_ms(10);
    hw32(abar, HBA_GHC, hr32(abar, HBA_GHC) | 0x8000_0000);

    hw32(abar, HBA_IS, 0xFFFF_FFFF);

    let pi = hr32(abar, HBA_PI);
    let mut found_port: Option<u8> = None;
    for p in 0u8..32 {
        if pi & (1 << p) == 0 { continue; }
        let pb = abar + 0x100 + p as u64 * 0x80;
        let ssts = pr32(pb, P_SSTS);
        let det = ssts & 0xF;
        let ipm = (ssts >> 8) & 0xF;
        if det == 3 && ipm == 1 {
            found_port = Some(p);
            break;
        }
    }
    let port = match found_port {
        Some(p) => p,
        None => {
            printk(b"ahci: no disk found\0".as_ptr());
            return false;
        }
    };
    let pb = abar + 0x100 + port as u64 * 0x80;

    let pcmd = pr32(pb, P_CMD);
    pw32(pb, P_CMD, pcmd & !(1 << 0));
    if !port_poll_clear(pb, P_CMD, 1 << 15, 500) {
        printk(b"ahci: port CR stuck\0".as_ptr());
        return false;
    }
    pw32(pb, P_CMD, pr32(pb, P_CMD) & !(1 << 4));
    if !port_poll_clear(pb, P_CMD, 1 << 14, 500) {
        printk(b"ahci: port FR stuck\0".as_ptr());
        return false;
    }

    let clb = alloc_phys_frame();
    let fb  = alloc_phys_frame();
    let ct  = alloc_phys_frame();
    zero_frame(clb);
    zero_frame(fb);
    zero_frame(ct);

    pw32(pb, P_CLB,  clb as u32);
    pw32(pb, P_CLBU, (clb >> 32) as u32);
    pw32(pb, P_FB,   fb as u32);
    pw32(pb, P_FBU,  (fb >> 32) as u32);

    pw32(pb, P_SERR, 0xFFFF_FFFF);
    pw32(pb, P_IS,   0xFFFF_FFFF);

    pw32(pb, P_CMD, pr32(pb, P_CMD) | (1 << 4));
    sleep_ms(1);
    pw32(pb, P_CMD, pr32(pb, P_CMD) | (1 << 0));

    printk(b"ahci: controller found, port init done\0".as_ptr());

    AHCI = Some(AhciState { abar, port_base: pb, clb, ct });
    true
}

#[no_mangle]
pub extern "C" fn ahci_read_sectors(lba: u64, count: u16, buf_phys: u64) -> i32 {
    let state = unsafe {
        match AHCI.as_ref() { Some(s) => s, None => return -1 }
    };
    let pb  = state.port_base;
    let clb = state.clb;
    let ct  = state.ct;
    let clb_virt = unsafe { phys_to_virt(clb) };
    let ct_virt  = unsafe { phys_to_virt(ct) };

    unsafe {
        core::ptr::write_bytes(clb_virt as *mut u8, 0, 32);
        core::ptr::write_bytes(ct_virt  as *mut u8, 0, 256);
    }

    let dw0: u32 = 5 | (1 << 16);
    unsafe {
        wmem32(clb_virt, 0, dw0);
        wmem32(clb_virt, 4, 0);
        wmem32(clb_virt, 8,  ct as u32);
        wmem32(clb_virt, 12, (ct >> 32) as u32);
    }

    unsafe {
        wmem8(ct_virt, 0, 0x27);
        wmem8(ct_virt, 1, 0x80);
        wmem8(ct_virt, 2, 0x25);
        wmem8(ct_virt, 3, 0x00);

        wmem8(ct_virt, 4, lba as u8);
        wmem8(ct_virt, 5, (lba >> 8) as u8);
        wmem8(ct_virt, 6, (lba >> 16) as u8);
        wmem8(ct_virt, 7, 0x40);

        wmem8(ct_virt, 8,  (lba >> 24) as u8);
        wmem8(ct_virt, 9,  (lba >> 32) as u8);
        wmem8(ct_virt, 10, (lba >> 40) as u8);
        wmem8(ct_virt, 11, 0x00);

        wmem8(ct_virt, 12, count as u8);
        wmem8(ct_virt, 13, (count >> 8) as u8);
    }

    let byte_count = count as u32 * 512;
    unsafe {
        wmem32(ct_virt, 0x80, buf_phys as u32);
        wmem32(ct_virt, 0x84, (buf_phys >> 32) as u32);
        wmem32(ct_virt, 0x88, 0);
        wmem32(ct_virt, 0x8C, (byte_count - 1) | 0x8000_0000);
    }

    pw32(pb, P_IS, 0xFFFF_FFFF);
    pw32(pb, P_CI, 1);

    let deadline = unsafe { tsc_now_ns() } + 2_000_000_000u64;
    loop {
        let is_reg = pr32(pb, P_IS);
        if is_reg & 0x78000000 != 0 {
            unsafe { printk(b"ahci: DMA error\0".as_ptr()); }
            pw32(pb, P_IS, 0xFFFF_FFFF);
            return -1;
        }
        if pr32(pb, P_CI) & 1 == 0 { break; }
        if unsafe { tsc_now_ns() } > deadline {
            unsafe { printk(b"ahci: read timeout\0".as_ptr()); }
            return -1;
        }
        unsafe { sleep_ms(1) };
    }

    if pr32(pb, P_TFD) & 0x01 != 0 {
        unsafe { printk(b"ahci: task file error\0".as_ptr()); }
        return -1;
    }

    0
}

#[no_mangle]
pub static module_name: [u8; 5] = *b"ahci\0";

#[no_mangle]
pub extern "C" fn module_init() -> i32 {
    if !unsafe { ahci_init() } { return -1; }
    unsafe {
        ksym_export(b"ahci_read_sectors\0".as_ptr(),
                    ahci_read_sectors as *const () as usize);
    }
    0
}

#[no_mangle]
pub unsafe extern "C" fn module_exit() {
    AHCI = None;
    printk(b"ahci: unloaded\0".as_ptr());
}
