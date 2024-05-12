//!Implementation of [`TaskManager`]
use super::TaskControlBlock;
use crate::sync::UPSafeCell;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use lazy_static::*;
use core::usize::MAX;
use crate::timer::get_time_ms;

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
        let task_inner = task.inner_exclusive_access();

        drop(task_inner);
        self.ready_queue.push_back(task);
    }

    /// Add process back to ready queue based on Short Job First algorithm
    /// 代码思路参考了马思源学长的毕设
    #[allow(unused)]
    pub fn sjf_add(&mut self, task: Arc<TaskControlBlock>) {
        let task_inner = task.inner_exclusive_access();
        let runtime = task_inner.runtime;
        let running = task_inner.running;

        drop(task_inner);
        let mut index_to_insert = self.ready_queue.len();
        for (index, tcb) in self.ready_queue.iter().enumerate(){
            let tcb_inner = tcb.inner_exclusive_access();
            let runtime_ = tcb_inner.runtime;
            let running_ = tcb_inner.running;
            if running & !running_ {
                index_to_insert = index;
                break
            } 
            else if !running & running_ {
                continue
            }          
            if runtime < runtime_ {
                index_to_insert = index;
                break
            }
        }
        self.ready_queue.insert(index_to_insert, task);
    }

    /// Add process back to ready queue based on Shortest Remaining Time algorithm
    /// 代码思路参考了马思源学长的毕设
    #[allow(unused)]
    pub fn srt_add(&mut self, task: Arc<TaskControlBlock>) {
        let task_inner = task.inner_exclusive_access();
        let remain_runtime = task_inner.remain_runtime;

        drop(task_inner);
        for index in 0..self.ready_queue.len(){
            let tcb = self.ready_queue.get_mut(index).unwrap();
            let remain_runtime_ = tcb.inner_exclusive_access().remain_runtime;

            if remain_runtime < remain_runtime_ {
                self.ready_queue.insert(index, task);
                return
            }
        }
        self.ready_queue.push_back(task);

    }
    
    /// Add process back to ready queue based on Highest Response Ratio Next algorithm
    /// 代码思路参考了马思源学长的毕设
    #[allow(unused)]
    pub fn hrrn_add(&mut self, task: Arc<TaskControlBlock>) {
        let task_inner = task.inner_exclusive_access();
        let runtime = task_inner.runtime;
        let running = task_inner.running;
        let waiting_time = task_inner.task_waiting_time + get_time_ms() - task_inner.task_last_yield_time;

        drop(task_inner);
        let mut index_to_insert = self.ready_queue.len();
        for index in 0..self.ready_queue.len(){
            let tcb = self.ready_queue.get_mut(index).unwrap();
            let tcb_inner = tcb.inner_exclusive_access();
            let runtime_ = tcb_inner.runtime;
            let running_ = tcb_inner.running;
            let waiting_time_ = tcb_inner.task_waiting_time + get_time_ms() - tcb_inner.task_last_yield_time;
            drop(tcb_inner);

            if running & !running_ {
                index_to_insert = index;
                break
            } 
            else if !running & running_ {
                continue
            }          
            if (waiting_time / runtime) > (waiting_time_ / runtime_) {
                index_to_insert = index;
                break
            }
        }
        self.ready_queue.insert(index_to_insert, task);
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
        let mut pid = 0;
        let mut prio = 0;
        let big_stride = 16000; 
        let mut i = 0;
        let mut min_i = 0;
        
        for tcb in self.ready_queue.iter() {
            let tcb_inner = tcb.inner_exclusive_access();
            if tcb_inner.stride < min_stride{
                min_stride = tcb_inner.stride;
                prio = tcb_inner.prio;
                pid = tcb.pid.0;
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
    // TASK_MANAGER.exclusive_access().add(task);
    TASK_MANAGER.exclusive_access().srt_add(task);
}

/// Take a process out of the ready queue
pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    //trace!("kernel: TaskManager::fetch_task");
    TASK_MANAGER.exclusive_access().fetch()
}
