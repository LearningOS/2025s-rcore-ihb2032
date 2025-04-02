//! Process management syscalls
use crate::{
    syscall::{SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_TRACE, SYSCALL_YIELD},
    task::{exit_current_and_run_next, suspend_current_and_run_next, TASK_MANAGER},
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    TASK_MANAGER.update_syscall_times(SYSCALL_EXIT);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    TASK_MANAGER.update_syscall_times(SYSCALL_YIELD);
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    TASK_MANAGER.update_syscall_times(SYSCALL_GET_TIME);
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

// TODO: implement the syscall
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    TASK_MANAGER.update_syscall_times(SYSCALL_TRACE);
    match trace_request {
        0 => {
            let addr = id as *const u8;
            let value = unsafe { addr.read_volatile() };
            value as isize
        }
        1 => {
            let addr = id as *mut u8;
            let data_byte = data as u8;
            unsafe { addr.write_volatile(data_byte) };
            0
        }
        2 => {
            let current_task = TASK_MANAGER.get_current_task();
            current_task.syscall_counts[id] as isize
        }
        _ => -1,
    }
}
