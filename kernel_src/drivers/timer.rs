use crate::shell::shell::SHELL;
use core::sync::atomic::{AtomicU64, Ordering};
use x86_64::structures::idt::InterruptStackFrame;
use core::arch::x86_64::_rdtsc;
use spin::Once;
use x86_64::instructions::port::Port;

static TICK_COUNT: AtomicU64 = AtomicU64::new(0);
static TSC_FREQ: Once<u64> = Once::new();

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let ticks = TICK_COUNT.fetch_add(1, Ordering::Relaxed);
    if ticks % 10 == 0 {
        if let Some(mut guard) = SHELL.try_lock() {
            if let Some(shell) = guard.as_mut() {
                shell.cursor_update();
            }
        }
    }
    unsafe {
        crate::arch::pic::PICS
            .lock()
            .notify_end_of_interrupt(crate::arch::pic::InterruptIndex::Timer.as_u8());
    }
    crate::task::scheduler::tick();
}
pub fn tsc_now_ns() -> u64 {
    let Some(&freq) = TSC_FREQ.get() else { return 0; };
    if freq == 0 { return 0; }
    let cycles = unsafe { _rdtsc() };
    cycles / (freq / 1_000_000) * 1_000
}
pub fn sleep_ms(ms: u64) {
    let end = tsc_now_ns() + ms * 1_000_000;
    while tsc_now_ns() < end { core::hint::spin_loop(); }
}
pub fn calibrate_tsc() {
    const PIT_HZ: u64 = 1_193_182;
    const MS10: u64 = PIT_HZ / 100; // ≈11932 тика = 10 мс

    let freq = unsafe {
        let mut p43: Port<u8> = Port::new(0x43);
        let mut p42: Port<u8> = Port::new(0x42);
        let mut p61: Port<u8> = Port::new(0x61);

        let saved = p61.read();
        p61.write(saved & !0x03);
        p43.write(0xB0);
        p42.write((MS10 & 0xFF) as u8); 
        p42.write(((MS10>>8) & 0xFF) as u8);
        let t0 = _rdtsc();
        p61.write((saved & !0x02) | 0x01);
        while p61.read() & 0x20 == 0 { core::hint::spin_loop(); }
        let t1 = _rdtsc();
        p61.write(saved & !0x01);
        (t1 - t0) * 100
    };
    TSC_FREQ.call_once(|| freq);
    crate::println!("TSC: {} MHz", freq / 1_000_000);
}
