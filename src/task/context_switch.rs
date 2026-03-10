use core::arch::global_asm;

global_asm!(
    r#"
.globl context_switch
context_switch:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    pushfq

    mov [rdi], rsp
    mov rsp, rsi

    popfq
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    ret

.globl context_switch_user
context_switch_user:
    push rax
    push rbx
    push rcx
    push rdx
    push rsi
    push rdi
    push rbp
     push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15
    pushfq

    mov [rdi], rsp
    mov rsp, rsi

    popfq
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rbp
    pop rdi
    pop rsi
    pop rdx
    pop rcx
    pop rbx
    pop rax

    test qword ptr [rsp+8], 3
    jnz .return_to_user
    ret
.return_to_user:
    iretq
"#
);

unsafe extern "C" {
    pub fn context_switch(old_rsp: *mut u64, new_rsp: u64);
    pub fn context_switch_user(old_rsp: *mut u64, new_rsp: u64);
}
