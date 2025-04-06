//! Process management syscalls
use core::{mem::size_of, slice::from_raw_parts};

use crate::{
    mm::{translated_byte_buffer, PTEFlags, PageTable, VirtAddr},
    syscall::{SYSCALL_EXIT, SYSCALL_GET_TIME, SYSCALL_MMAP, SYSCALL_MUNMAP, SYSCALL_TRACE, SYSCALL_YIELD},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next, mmap, munmap, suspend_current_and_run_next, TASK_MANAGER
    },
    timer::get_time_us,
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

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    TASK_MANAGER.update_syscall_times(SYSCALL_GET_TIME);
    let time_us = get_time_us();
    let kernel_time = TimeVal {
        sec: time_us / 1_000_000,
        usec: time_us / 1_000_000,
    };
    let mut ptr = &kernel_time as *const TimeVal as usize;
    let mut buffers =
        translated_byte_buffer(current_user_token(), _ts as *const u8, size_of::<TimeVal>());
    for buffer in buffers.iter_mut() {
        let data = unsafe { from_raw_parts(ptr as *const u8, buffer.len()) };
        ptr += buffer.len();
        buffer.copy_from_slice(data);
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    TASK_MANAGER.update_syscall_times(SYSCALL_TRACE);
    match _trace_request {
        0 => {
            let vpn = VirtAddr::from(_id as *const u8 as usize).floor();
            if let Some(pte) = PageTable::from_token(current_user_token()).translate(vpn) {
                let flags = pte.flags();
                if pte.is_valid() && (flags & PTEFlags::U) != PTEFlags::empty() && pte.readable() {
                    let buffers = translated_byte_buffer(
                        current_user_token(),
                        _id as *const u8,
                        size_of::<u8>(),
                    );
                    return buffers[0][0] as isize;
                }
            }
            -1
        }
        1 => {
            let vpn = VirtAddr::from(_id as *mut u8 as usize).floor();
            if let Some(pte) = PageTable::from_token(current_user_token()).translate(vpn) {
                let flags = pte.flags();
                if pte.is_valid() && (flags & PTEFlags::U) != PTEFlags::empty() && pte.writable() {
                    let mut buffers = translated_byte_buffer(
                        current_user_token(),
                        _id as *mut u8,
                        size_of::<u8>(),
                    );
                    buffers[0][0] = _data as u8;
                    return 0;
                }
            }
            -1
        }
        2 => TASK_MANAGER.get_syscall_counts(_id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    TASK_MANAGER.update_syscall_times(SYSCALL_MMAP);
    mmap(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    TASK_MANAGER.update_syscall_times(SYSCALL_MUNMAP);
munmap(_start, _len)
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
