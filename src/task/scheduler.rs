extern crate alloc;
use super::task::{Task, TaskState};
use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

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
        self.tasks.push(Task::new(id, entry));
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
    fn prepare_switch(&mut self) -> Option<(*mut u64, u64)> {
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
        self.tasks[self.current].state = TaskState::Ready;
        self.tasks[next].state = TaskState::Running;

        let old_rsp = &mut self.tasks[self.current].stack_pointer as *mut _ as *mut u64;
        let new_rsp = self.tasks[next].stack_pointer.as_u64();
        self.current = next;
        self.remaining_ticks = TIME_SLICE;

        Some((old_rsp, new_rsp))
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
    SCHEDULER.lock().yielded = true;
}

pub fn tick() {
    let switch_data = { SCHEDULER.lock().prepare_switch() };
    if let Some((old_rsp, new_rsp)) = switch_data {
        unsafe {
            super::context_switch::context_switch(old_rsp, new_rsp);
        }
    }
}
pub fn start() {
    let (old_rsp, new_rsp) = { SCHEDULER.lock().prepare_start() };
    unsafe {
        super::context_switch::context_switch(old_rsp, new_rsp);
    }
}
