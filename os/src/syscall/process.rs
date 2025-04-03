//! Process management syscalls
use core::{mem::size_of, slice::from_raw_parts};

use crate::{
    config::PAGE_SIZE,
    mm::{memory_set::MapType, translated_byte_buffer, MapPermission, VirtAddr},
    syscall::{SYSCALL_GET_TIME, SYSCALL_TRACE},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next, TASK_MANAGER,
    },
    timer::get_time,
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
    TASK_MANAGER.update_syscall_times(SYSCALL_GET_TIME);
    let kernel_time = TimeVal {
        sec: get_time() / 1_000_000,
        usec: get_time() / 1_000_000,
    };
    let time_bytes =
        unsafe { from_raw_parts(&kernel_time as *const _ as *const u8, size_of::<TimeVal>()) };
    let buffers = translated_byte_buffer(current_user_token(), _ts as *const u8, time_bytes.len());
    let total_cap: usize = buffers.iter().map(|b| b.len()).sum();
    if total_cap < buffers.len() {
        return -1;
    }
    let mut copied = 0;
    for buffer in buffers {
        let copy_len = buffer.len().min(time_bytes.len() - copied);
        buffer[..copy_len].copy_from_slice(&time_bytes[copied..copied + copy_len]);
        copied += copy_len;
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
        2 => TASK_MANAGER.get_syscall_counts(_id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 {
        return -1;
    }
    if (_port & !0x7) != 0 || (_port & 0x7) == 0 {
        return -1;
    }
    let len = if _len == 0 {
        0
    } else {
        ((_len + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE
    };
    if len == 0 {
        return 0;
    }
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + len);
    let current_task_id = TASK_MANAGER.inner.exclusive_access().current_task;
    let memory_set = &mut TASK_MANAGER.inner.exclusive_access().tasks[current_task_id].memory_set;
    let mut map_perm = MapPermission::U;
    if _port & 0x1 != 0 {
        map_perm = MapPermission::R;
    }
    if _port & 0x2 != 0 {
        map_perm = MapPermission::W;
    }
    if _port & 0x3 != 0 {
        map_perm = MapPermission::X;
    }
    memory_set.insert_framed_area(start_va, end_va, map_perm);
    0
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    if _start % PAGE_SIZE != 0 || _len % PAGE_SIZE != 0 {
        return -1;
    }
    let len = (_len + PAGE_SIZE - 1) & !(PAGE_SIZE - 1);
    if len == 0 {
        return 0;
    }
    let start_va = VirtAddr::from(_start);
    let end_va = VirtAddr::from(_start + len);
    let current_task_id = TASK_MANAGER.inner.exclusive_access().current_task;
    let memory_set = &mut TASK_MANAGER.inner.exclusive_access().tasks[current_task_id].memory_set;
    let pos = memory_set.areas.iter().position(|area| {
        area.map_type == MapType::Framed
            && area.vpn_range.get_start().0 == start_va.0
            && area.vpn_range.get_end().0 == end_va.0
    });
    let pos = match pos {
        Some(p) => p,
        None => return -1,
    };
    let area = memory_set.areas.swap_remove(pos);
    let page_table = &mut memory_set.page_table;
    for vpn in area.vpn_range.into_iter() {
        page_table.unmap(vpn);
    }
    0
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
