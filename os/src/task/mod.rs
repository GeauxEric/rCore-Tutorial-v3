mod context;
mod switch;
mod task;

use crate::config::{APP_SIZE_LIMIT, BIG_STRIDE, MAX_APP_NUM};
use crate::loader::{get_num_app, init_app_cx};
use lazy_static::*;
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};
use crate::sync::UPSafeCell;
use crate::loader::get_base_i;

pub use context::TaskContext;

pub struct TaskManager {
    num_app: usize,
    inner: UPSafeCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock {
                task_cx: TaskContext::zero_init(),
                task_status: TaskStatus::UnInit,
                priority: 16,
                stride: 0,
            };
            MAX_APP_NUM
        ];
        for i in 0..num_app {
            tasks[i].task_cx = TaskContext::goto_restore(init_app_cx(i));
            tasks[i].task_status = TaskStatus::Ready;
        }
        TaskManager {
            num_app,
            inner: unsafe { UPSafeCell::new(TaskManagerInner {
                tasks,
                current_task: 0,
            })},
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let mut inner = self.inner.exclusive_access();
        let task0 = &mut inner.tasks[0];
        task0.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0.task_cx as *const TaskContext;
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        // before this, we should drop local variables that must be dropped manually
        unsafe {
            __switch(
                &mut _unused as *mut TaskContext,
                next_task_cx_ptr,
            );
        }
        panic!("unreachable in run_first_task!");
    }

    fn set_priority(&self, priority: isize) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].priority = priority;
    }

    fn get_current_address_range(&self) -> (usize, usize){
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let base = get_base_i(current);
        (base, base + APP_SIZE_LIMIT)
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.exclusive_access();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let mut min_stride = usize::MAX;
        let mut min_stride_tid = self.num_app + 1;
        for tid in 0..self.num_app {
            let task = inner.tasks[tid];
            if task.task_status != TaskStatus::Ready {
                continue
            }
            if task.stride < min_stride {
                min_stride = task.stride;
                min_stride_tid = tid;
            }
        }
        if min_stride_tid == self.num_app + 1 {
            return None;
        } else {
            let tid = min_stride_tid;
            let mut task = inner.tasks[tid];
            task.stride += (BIG_STRIDE / task.priority) as usize;
            return Some(tid)
        }
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.current_task = next;
            let current_task_cx_ptr = &mut inner.tasks[current].task_cx as *mut TaskContext;
            let next_task_cx_ptr = &inner.tasks[next].task_cx as *const TaskContext;
            drop(inner);
            // before this, we should drop local variables that must be dropped manually
            unsafe {
                __switch(
                    current_task_cx_ptr,
                    next_task_cx_ptr,
                );
            }
            // go back to user mode
        } else {
            panic!("All applications completed!");
        }
    }
}

pub fn get_current_task_address_range() -> (usize, usize) {
    TASK_MANAGER.get_current_address_range()
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn set_priority(priority: isize) {
    TASK_MANAGER.set_priority(priority)
}

pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}