//! Process management syscalls
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    loader::get_app_data_by_name,
    mm::{translated_refmut, translated_str},
    task::{
        add_task, current_task, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TaskStatus, set_task_running,
    },
    timer::{get_time_ms,get_time_us},
};
use crate::config::PAGE_SIZE;

use crate::mm::{MapPermission, PageTable, VirtAddr, VirtPageNum, PhysAddr};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    set_task_running(false);
    suspend_current_and_run_next();
    0
}

pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8, runtime: usize) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data, runtime);
        let mut task_inner = task.inner_exclusive_access();
        task_inner.remain_runtime -= (get_time_ms() - task_inner.task_last_start_time) as isize;
        drop(task_inner);
        suspend_current_and_run_next();
        0
    } else {
        suspend_current_and_run_next();
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    trace!("kernel::pid[{}] sys_waitpid [{}]", current_task().unwrap().pid.0, pid);
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let va = VirtAddr(_ts as usize);
    let offset = va.page_offset();
    let vpn = va.floor();
    let ppn = PageTable::from_token(current_user_token()).translate(vpn).map(|entry| entry.ppn()).unwrap();
    let pa = PhysAddr(ppn.0 * PAGE_SIZE + offset);
    
    let us = get_time_us();
    let ts = pa.0 as *mut TimeVal;
    unsafe {
            *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let va = VirtAddr(_ti as usize);
    let offset = va.page_offset();
    let vpn = va.floor();
    let ppn = PageTable::from_token(current_user_token()).translate(vpn).map(|entry| entry.ppn()).unwrap();
    let pa = PhysAddr(ppn.0 * PAGE_SIZE + offset);

    let ti = pa.0 as *mut TaskInfo;
    current_task().unwrap().get_current_task_info(ti);
    0
}


/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    if _start & (PAGE_SIZE - 1) != 0 {
        println!("the start address should be page-aligned");
        return -1;
    }
    if _port & !0x7 != 0{
        println!("the remaining bits of the port must be zero");
        return -1;
    }
    if _port & 0x7 == 0{
        println!("this memory is meaningless");
        return -1;
    }
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let memory_set = &mut inner.memory_set;
    let start_vpn = VirtPageNum::from(VirtAddr(_start).floor());
    let end_vpn = VirtPageNum::from(VirtAddr(_start + _len).ceil());
    for vpn in start_vpn.0 .. end_vpn.0 {
        if let Some(pte) = memory_set.translate(VirtPageNum(vpn)) {
            if pte.is_valid() {
                println!("The page has already been mapped");
                return -1;
            }   }
    }

    let permission = MapPermission::from_bits((_port as u8) << 1).unwrap() | MapPermission::U;
    memory_set.insert_framed_area(VirtAddr(_start), VirtAddr(_start + _len), permission);
    0
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    if _start & (PAGE_SIZE - 1) != 0 {
        println!("the start address should be page-aligned");
        return -1;
    }

    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    let memory_set = &mut inner.memory_set;
    let start_vpn = VirtPageNum::from(VirtAddr(_start).floor());
    let end_vpn = VirtPageNum::from(VirtAddr(_start + _len).ceil());
    for vpn in start_vpn.0 .. end_vpn.0 {
        if let Some(pte) = memory_set.translate(VirtPageNum(vpn)) {
            if !pte.is_valid() {
                println!("The page hasn't been mapped");
                return -1;
            }
        }
    }
    memory_set.delete_framed_area(VirtAddr(_start), VirtAddr(_start + _len));
    0
}

/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(_path: *const u8,runtime: usize) -> isize {
    let token = current_user_token();
    let path = translated_str(token, _path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let current_task = current_task().unwrap();
        let new_task = current_task.spawn(data, runtime);
        let new_pid = new_task.pid.0;
        // modify trap context of new_task, because it returns immediately after switching
        let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
        trap_cx.x[10] = 0;
        // add new task to scheduler
        add_task(new_task);
        suspend_current_and_run_next();
        new_pid as isize
    } else {
        suspend_current_and_run_next();
        -1
    }
}

// YOUR JOB: Set task priority.
pub fn sys_set_priority(_prio: isize) -> isize {
    if _prio >= 2 {
        current_task().unwrap().inner_exclusive_access().prio = _prio;
        _prio
    } else {
        -1
    }
}
