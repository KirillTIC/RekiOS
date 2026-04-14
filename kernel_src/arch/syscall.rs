use crate::print;
use core::arch::global_asm;
use x86_64::{
    VirtAddr,
    registers::{
        model_specific::{Efer, EferFlags, LStar, SFMask, Star},
        rflags::RFlags,
    },
    structures::gdt::SegmentSelector,
};

global_asm!(
    r#"
.globl syscall_entry
syscall_entry:
    # SYSCALL: RCX = user RIP, R11 = user RFLAGS
    # Save user registers
    push rcx
    push r11
    push rbp
    push rbx
    push r12
    push r13
    push r14
    push r15
    push rdi
    push rsi
    push rdx

    # Shuffle args for C calling convention:
    # syscall convention: rax=num, rdi=arg1, rsi=arg2, rdx=arg3
    # C calling convention: rdi=param0, rsi=param1, rdx=param2, rcx=param3
    mov rcx, rdx
    mov rdx, rsi
    mov rsi, rdi
    mov rdi, rax

    # Align stack to 16 bytes for the call
    mov rbp, rsp
    and rsp, -16
    call syscall_dispatch
    mov rsp, rbp

    # rax = return value

    pop rdx
    pop rsi
    pop rdi
    pop r15
    pop r14
    pop r13
    pop r12
    pop rbp
    pop rbx
    pop r11
    pop rcx

    sysretq
"#
);

unsafe extern "C" {
    fn syscall_entry();
}

pub fn init_syscalls(
    kernel_code_selector: SegmentSelector,
    kernel_data_selector: SegmentSelector,
    user_code_selector: SegmentSelector,
    user_data_selector: SegmentSelector,
) {
    unsafe {
        Efer::update(|f| *f |= EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
    LStar::write(VirtAddr::new(syscall_entry as *const () as u64));
    Star::write(
        user_code_selector,
        user_data_selector,
        kernel_code_selector,
        kernel_data_selector,
    )
    .unwrap();
    SFMask::write(RFlags::INTERRUPT_FLAG);
}

#[unsafe(no_mangle)]
extern "C" fn syscall_dispatch(num: u64, arg1: u64, arg2: u64, arg3: u64) -> i64 {
    match num {
        0 => sys_write(arg1, arg2, arg3),
        1 => sys_exit(arg1 as i64),
        _ => -1,
    }
}

fn sys_write(fd: u64, buf_ptr: u64, len: u64) -> i64 {
    if fd != 1 {
        return -1;
    }
    if buf_ptr >= 0xFFFF_8000_0000_0000 {
        return -1;
    }
    let slice = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len as usize) };
    if let Ok(s) = core::str::from_utf8(slice) {
        print!("{}", s);
    }
    len as i64
}

fn sys_exit(code: i64) -> i64 {
    if code != 0 {
        print!("Process exited with code {}\n", code);
    }
    crate::task::scheduler::SCHEDULER.lock().mark_current_dead();
    crate::task::scheduler::yield_now();
    x86_64::instructions::interrupts::enable();
    loop {
        x86_64::instructions::hlt();
    }
}
