//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, get_first_scheduled_time, get_syscall_times, suspend_current_and_run_next },
    timer::{get_time_ms, get_time_us},
};

pub use crate::task::TaskInfo;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    unsafe {
        (*_ti).status = crate::task::TaskStatus::Running;
        (*_ti).time = get_time_ms() - get_first_scheduled_time();
        get_syscall_times().iter()
            .enumerate()
            .filter(|(_, &v)| v != 0)
            .for_each(|(i, &times)| (*_ti).syscall_times[i] = times);
    }
    0
}
