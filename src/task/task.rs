extern crate alloc;
use core::usize;

use alloc::boxed::Box;
use x86_64::{VirtAddr, structures::paging::PhysFrame};

use crate::task::{idle::idle_task, scheduler::SCHEDULER};

const STACK_SIZE: usize = 4096 * 4;

pub struct Task {
    pub id: usize,
    pub state: TaskState,
    pub stack_pointer: VirtAddr,
    pub p4_frame: Option<PhysFrame>,
    _stack: Box<[u8; STACK_SIZE]>,
}

#[derive(PartialEq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Dead,
}
impl Task {
    pub fn new_user(id: usize, entry_point: u64, p4_frame: PhysFrame) -> Self {
        Self::new_user_with_args(id, entry_point, p4_frame, 0, 0)
    }
    pub fn new_user_with_args(
        id: usize,
        entry_point: u64,
        p4_frame: PhysFrame,
        rdi: u64,
        rsi: u64,
    ) -> Self {
        let stack = Box::new([0u8; STACK_SIZE]);
        let stack_top = stack.as_ptr() as usize + STACK_SIZE;
        let stack_top = stack_top & !0xF;
        let rsp = Self::init_stack(stack_top, entry_point, rdi, rsi);

        Self {
            id,
            state: TaskState::Ready,
            stack_pointer: VirtAddr::new(rsp as u64),
            p4_frame: Some(p4_frame),
            _stack: stack,
        }
    }
    pub fn new_kernel_with_entry(id: usize, entry_point: fn() -> !) -> Self {
        let stack = Box::new([0u8; STACK_SIZE]);
        let stack_top = stack.as_ptr() as usize + STACK_SIZE;
        let stack_top = stack_top & !0xF;
        let rsp = Self::init_stack(stack_top, entry_point as u64, 0, 0);

        Self {
            id,
            state: TaskState::Ready,
            stack_pointer: VirtAddr::new(rsp as u64),
            p4_frame: None,
            _stack: stack,
        }
    }
    fn init_stack(stack_top: usize, entry: u64, rdi: u64, rsi: u64) -> usize {
        let mut rsp = stack_top;

        unsafe fn push(rsp: &mut usize, val: u64) {
            *rsp -= 8;
            unsafe {
                *(*rsp as *mut u64) = val;
            }
        }

        unsafe {
            push(&mut rsp, 0);          // alignment
            push(&mut rsp, entry);      // return address
            push(&mut rsp, 0);          // rax
            push(&mut rsp, 0);          // rbx
            push(&mut rsp, 0);          // rcx
            push(&mut rsp, 0);          // rdx
            push(&mut rsp, rsi);        // rsi
            push(&mut rsp, rdi);        // rdi
            push(&mut rsp, 0);          // rbp
            push(&mut rsp, 0);          // r8
            push(&mut rsp, 0);          // r9
            push(&mut rsp, 0);          // r10
            push(&mut rsp, 0);          // r11
            push(&mut rsp, 0);          // r12
            push(&mut rsp, 0);          // r13
            push(&mut rsp, 0);          // r14
            push(&mut rsp, 0);          // r15
            push(&mut rsp, 0x200);      // RFLAGS
        }

        rsp
    }
    pub fn new_kernel() -> Self {
        Self {
            id: usize::MAX,
            state: TaskState::Running,
            stack_pointer: VirtAddr::new(0),
            p4_frame: None,
            _stack: Box::new([0u8; STACK_SIZE]),
        }
    }
}
pub fn init() {
    let mut scheduler = SCHEDULER.lock();
    scheduler.add_task(idle_task);
}
