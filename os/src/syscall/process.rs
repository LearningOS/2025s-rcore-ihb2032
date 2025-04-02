//! Process management syscalls
use crate::{
    mm::translated_byte_buffer, syscall::SYSCALL_TRACE, task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TASK_MANAGER,
    }
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
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
    -1
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    TASK_MANAGER.update_syscall_times(SYSCALL_TRACE);
    match _trace_request {
        0 => {
            let buffers = translated_byte_buffer(current_user_token(), _id as *const u8, 1);
            if buffers.iter().map(|b| b.len()).sum::<usize>() < 1 {
                return -1;
            }
            let mut value = 0u8;
            let mut remaining = 1;
            for buffer in buffers {
                let copy_len = buffer.len().min(remaining);
                value = buffer[0];
                remaining = copy_len;
                if remaining == 0 {
                    break;
                }
            }
            value as isize
        }
        1 => {
            let data_byte = _data as u8;
            let buffers = translated_byte_buffer(current_user_token(), _id as *const u8, 1);
            if buffers.iter().map(|b| b.len()).sum::<usize>() < 1 {
                return -1;
            }
            let mut remaining = 1;
            for buffer in buffers {
                let copy_len = buffer.len().min(remaining);
                buffer[..copy_len].fill(data_byte);
                remaining = copy_len;
                if remaining == 0 {
                    break;
                }
            }
            0
        }
        2 => {
            let current_task = TASK_MANAGER.get_current_task();
            current_task.syscall_counts[_id] as isize
        }
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
