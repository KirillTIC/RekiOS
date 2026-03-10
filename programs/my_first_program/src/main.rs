#![no_std]
#![no_main]

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        core::arch::asm!(
            "mov rsi, {msg}",
            "mov rax, 0",
            "mov rdi, 1",
            "mov rdx, 21",
            "syscall",
            msg = in(reg) b"Hello from userspace!".as_ptr(),
        );
        core::arch::asm!("mov rax, 1", "mov rdi, 0", "syscall",);
    }
    loop {}
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
