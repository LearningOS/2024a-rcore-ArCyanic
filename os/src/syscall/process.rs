//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, 
    mm::{copy_data_to_va, MapPermission, PageTable, VirtAddr, OverlapType}, 
    task::{
        change_program_brk, current_memory_set, current_user_token, exit_current_and_run_next, get_first_invoked_time, get_syscall_times, suspend_current_and_run_next, TaskStatus
    }, 
    timer::{get_time_ms, get_time_us}
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
#[repr(C)]
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    // TODO: Waited to improve!
    let us = get_time_us();

    let token = current_user_token();
    let page_table = PageTable::from_token(token);
    let start_va = VirtAddr::from(_ts as usize);
    let vpn = start_va.floor();
    let ppn = page_table.translate(vpn).unwrap().ppn();
    unsafe {
        let pa = &mut ppn.get_bytes_array()[start_va.page_offset()] as *mut u8 as *mut usize;
        *pa.add(0) = us / 1_000_000;
        *(pa.add(1)) = us % 1_000_000;
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");

    // create a new TaskInfo 
    let ti = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_syscall_times(),
        time: get_time_ms() - get_first_invoked_time()
    };

    let data = unsafe {
        core::slice::from_raw_parts(
            &ti as *const TaskInfo as *const u8, 
            core::mem::size_of::<TaskInfo>())
    };

    copy_data_to_va(current_user_token(), _ti as usize, data);
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap ");
    let memory_set = unsafe { &mut *current_memory_set() };
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + _len);
    if _port & !0x7 != 0 
    || _port & 0x7 == 0
    || memory_set.overlap(start_va, end_va) != OverlapType::None 
    || !start_va.aligned() {
        return -1;
    }
    // TODO: add more error detection
    let mut permission = MapPermission::U;
    if _port & 1 != 0 { permission |= MapPermission::R; }
    if _port & 2 != 0 { permission |= MapPermission::W; }
    if _port & 4 != 0 { permission |= MapPermission::X; }
    memory_set.insert_framed_area(
        VirtAddr::from(_start), 
        VirtAddr::from(_start + _len), 
        permission);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    let memory_set = unsafe { &mut *current_memory_set() };
    let start_va = VirtAddr::from(_start);
    if !start_va.aligned() { return -1; }
    let end_va = VirtAddr::from(_start + _len);
    match memory_set.overlap(start_va, end_va) {
        OverlapType::Covered(idx) => {
            memory_set.remove_framed_area(start_va, end_va, idx);
            return 0;
        }
        _ => { return -1; }
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
