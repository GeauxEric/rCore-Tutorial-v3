use fs::*;
use process::*;

use crate::task::get_current_task_address_range;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;

mod fs;
mod process;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => {
            // TODO: get user app address space
            let (base, ceil) = get_current_task_address_range();
            let err_ret: isize = -1;
            if args[1] < base || args[1] >= ceil {
                err_ret
            } else if args[1] + args[2] >= ceil {
                err_ret
            } else {
                sys_write(args[0], args[1] as *const u8, args[2])
            }
        },
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

