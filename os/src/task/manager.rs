//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use core::usize::MAX;
///A array of `TaskControlBlock` that is thread-safe
pub struct TaskManager {
    ready_queue: VecDeque<Arc<TaskControlBlock>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    ///Creat an empty TaskManager
    pub fn new() -> Self {
        Self {
            ready_queue: VecDeque::new(),
        }
    }
    /// Add process back to ready queue
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push_back(task);
    }
    /// Take a process out of the ready queue
    #[allow(unused)]
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop_front()
    }
    /// Take a process out of the ready queue using stride algorithm
    #[allow(unused)]
    pub fn stride_fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        if self.ready_queue.is_empty() {
            return None;
        }
        let mut min_stride = MAX;
        let mut prio = 0;
        let big_stride = 16000; 
        let mut i = 0;
        let mut min_i = 0;
        for tcb in self.ready_queue.iter() {
            let tcb_inner = tcb.inner_exclusive_access();
            if tcb_inner.stride < min_stride {
                min_stride = tcb_inner.stride;
                prio = tcb_inner.prio;
                min_i = i;
            }  
            i += 1;
        }
        self.ready_queue[min_i].inner_exclusive_access().stride += big_stride / (prio as usize); 
        self.ready_queue.remove(min_i)
    }
}

lazy_static! {
    /// TASK_MANAGER instance through lazy_static!
    pub static ref TASK_MANAGER: UPSafeCell<TaskManager> =
        unsafe { UPSafeCell::new(TaskManager::new()) };
}

/// Add process to ready queue
pub fn add_task(task: Arc<TaskControlBlock>) {
    //trace!("kernel: TaskManager::add_task");
    TASK_MANAGER.exclusive_access().add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
    // TASK_MANAGER.exclusive_access().stride_fetch()
}
