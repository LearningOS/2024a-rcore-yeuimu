//! Process management syscalls
#[allow(unused)]
use crate::{
    config::MAX_SYSCALL_NUM,
    mm::{VirtAddr, VirtPageNum, PhysAddr},
    task::{
        change_program_brk, exit_current_and_run_next, get_first_running_time, get_syscall_info,
        mmap_current_task, unmap_current_task, suspend_current_and_run_next, translate, TaskStatus,
    },
    timer::{get_time_ms, get_time_us},
};

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
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
#[allow(unused)]
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();

    let va = VirtAddr::from(ts as usize);
    let va_offer = va.page_offset();
    let vpn = va.floor();
    let ppn = translate(vpn).unwrap().ppn();
    let pa = PhysAddr::from(ppn);
    let ts_pa = PhysAddr::from(pa.0 | va_offer);
    let ts = ts_pa.get_mut() as *mut TimeVal;

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
#[allow(unused)]
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info NOT IMPLEMENTED YET!");
    let syscall_items = get_syscall_info();
    let time = get_first_running_time();
    if syscall_items.len() == 0 || time == 0 {
        return -1;
    }

    let va = VirtAddr::from(ti as usize);
    let va_offer = va.page_offset();
    let vpn = va.floor();
    let ppn = translate(vpn).unwrap().ppn();
    let pa = PhysAddr::from(ppn);
    let ti_pa = PhysAddr::from(pa.0 | va_offer);
    let ti = ti_pa.get_mut() as *mut TaskInfo;

    unsafe {
        (*ti).status = TaskStatus::Running;
        (*ti).syscall_times = syscall_items;
        (*ti).time = get_time_ms() as usize - time;
    }
    0
}

// YOUR JOB: Implement mmap.
#[allow(unused)]
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    match mmap_current_task(start, len, port) {
        Ok(_) => 0,
        Err(str) => {
            error!("{}", str);
            -1
        }
    }
}

// YOUR JOB: Implement munmap.
#[allow(unused)]
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    match unmap_current_task(start, len) {
        Ok(_) => 0,
        Err(str) => {
            error!("{}", str);
            -1
        }
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
