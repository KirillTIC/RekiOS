extern crate alloc;
use super::task::{Task, TaskState};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::{Mutex, Once};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::PhysFrame;

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

const TIME_SLICE: usize = 5;

pub struct Scheduler {
    tasks: Vec<Task>,
    current: usize,
    next_id: usize,
    remaining_ticks: usize,
    yielded: bool,
}
impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current: 0,
            next_id: 0,
            remaining_ticks: TIME_SLICE,
            yielded: false,
        }
    }
    pub fn add_task(&mut self, entry: fn() -> !) {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task::new_kernel_with_entry(id, entry));
    }
    pub fn add_user_task(&mut self, entry: u64, p4_frame: PhysFrame) {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks.push(Task::new_user(id, entry, p4_frame))
    }
    pub fn add_user_task_with_args(&mut self, entry: u64, p4_frame: PhysFrame, rdi: u64, rsi: u64) {
        let id = self.next_id;
        self.next_id += 1;
        self.tasks
            .push(Task::new_user_with_args(id, entry, p4_frame, rdi, rsi))
    }
    pub fn mark_current_dead(&mut self) {
        self.tasks[self.current].state = TaskState::Dead;
    }
    pub fn cleanup_dead(&mut self) {
        let current_id = self.tasks.get(self.current).map(|t| t.id);
        self.tasks.retain(|t| t.state != TaskState::Dead);
        if let Some(id) = current_id {
            self.current = self.tasks.iter().position(|t| t.id == id).unwrap_or(0);
        }
    }
    fn next_task(&self) -> Option<usize> {
        let len = self.tasks.len();

        if len == 0 {
            return None;
        }

        let mut idx = (self.current + 1) % len;
        for _ in 0..len {
            if self.tasks[idx].state == TaskState::Ready
                || self.tasks[idx].state == TaskState::Running
            {
                return Some(idx);
            }
            idx = (idx + 1) % len;
        }
        None
    }
    fn prepare_switch(&mut self) -> Option<(*mut u64, u64, bool, Option<PhysFrame>)> {
        if self.tasks.is_empty() {
            return None;
        }
        if !self.yielded {
            self.remaining_ticks = self.remaining_ticks.saturating_sub(1);
            if self.remaining_ticks > 0 {
                return None;
            }
        }
        self.yielded = false;
        let next = match self.next_task() {
            Some(n) => n,
            None => return None,
        };
        if next == self.current {
            self.remaining_ticks = TIME_SLICE;
            return None;
        }
        if self.tasks[self.current].state != TaskState::Dead {
            self.tasks[self.current].state = TaskState::Ready;
        }
        self.tasks[next].state = TaskState::Running;

        let old_rsp = &mut self.tasks[self.current].stack_pointer as *mut _ as *mut u64;
        let new_rsp = self.tasks[next].stack_pointer.as_u64();
        let is_user = self.tasks[next].p4_frame.is_some();
        let p4_frame = self.tasks[next].p4_frame;
        self.current = next;
        self.remaining_ticks = TIME_SLICE;

        Some((old_rsp, new_rsp, is_user, p4_frame))
    }
    fn prepare_start(&mut self) -> (*mut u64, u64) {
        assert!(!self.tasks.is_empty(), "No tasks for scheduler");

        let kernel_task = Task::new_kernel();
        self.tasks.insert(0, kernel_task);
        self.tasks[1].state = TaskState::Running;

        let old_rsp = &mut self.tasks[0].stack_pointer as *mut _ as *mut u64;
        let new_rsp = self.tasks[1].stack_pointer.as_u64();
        self.current = 1;

        (old_rsp, new_rsp)
    }
}
pub fn yield_now() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        SCHEDULER.lock().yielded = true;
    });
}

pub fn cleanup_dead_and_yield() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        if let Some(mut scheduler) = SCHEDULER.try_lock() {
            scheduler.cleanup_dead();
            scheduler.yielded = true;
        }
    });
}

static KERNEL_CR3: Once<PhysFrame> = Once::new();

pub fn tick() {
    let switch_data = { SCHEDULER.lock().prepare_switch() };
    if let Some((old_rsp, new_rsp, is_user, p4_frame)) = switch_data {
        unsafe {
            if let Some(frame) = p4_frame {
                KERNEL_CR3.call_once(|| Cr3::read().0);
                Cr3::write(frame, Cr3Flags::empty());
            } else if KERNEL_CR3.get().is_some() {
                Cr3::write(*KERNEL_CR3.get().unwrap(), Cr3Flags::empty());
            }

            if is_user {
                super::context_switch::context_switch_user(old_rsp, new_rsp);
            } else {
                super::context_switch::context_switch(old_rsp, new_rsp);
            }
        }
    }
}
pub fn start() {
    let (old_rsp, new_rsp) = { SCHEDULER.lock().prepare_start() };
    unsafe {
        super::context_switch::context_switch(old_rsp, new_rsp);
    }
}
