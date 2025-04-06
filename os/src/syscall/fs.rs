//! File and filesystem-related syscalls

use crate::mm::translated_byte_buffer;
use crate::syscall::SYSCALL_WRITE;
use crate::task::{current_user_token, TASK_MANAGER};

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel: sys_write");
    TASK_MANAGER.update_syscall_times(SYSCALL_WRITE);
    match fd {
        FD_STDOUT => {
            let buffers = translated_byte_buffer(current_user_token(), buf, len);
            for buffer in buffers {
                print!("{}", core::str::from_utf8(buffer).unwrap());
            }
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
