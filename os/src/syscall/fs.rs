use crate::fs::make_pipe;
use crate::mm::{translated_byte_buffer, translated_refmut, UserBuffer};
use crate::task::{current_task, current_user_token, get_task};

/// Send message to the mailbox of a process
///
/// # Arguments
/// * `pid` - target process whose mailbox to receive the message. If the mailbox is full, returns -1.
/// * `buf` - message content start
/// * `len` - length of the message. If greater than 256, the message is cut off at 256.
/// if len = 0, do not write and returns 0 if the target mailbox isn't full.
pub fn sys_mailwrite(pid: usize, buf: *const u8, len: usize) -> isize {
    let target_task = {
        let current = current_task().unwrap();
        if current.pid.0 == pid {
            Some(current)
        } else {
            get_task(pid)
        }
    };
    if target_task.is_none() {
        return -1;
    }
    let _task = target_task.unwrap();
    let inner = _task.inner_exclusive_access();
    if let Some(file) = inner.fd_table[3].as_ref() {
        let file = file.clone();
        drop(inner);

        let len = 256.min(len);
        if file.is_full() {
            return  -1;
        }
        if len == 0 {
            return 0;
        }

        let token = current_user_token();
        let len = 256.min(len);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

/// Read a message from the mailbox of the current process and write to a buf
///
/// # Arguments
/// * `buf` - start of the buf
pub fn sys_mailread(buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if let Some(file) = inner.fd_table[3].as_ref() {
        let file = file.clone();
        drop(inner);
        if file.is_empty() {
            return -1;
        }
        let len = 256.min(len);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        return -1;
    }
}

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.read(
            UserBuffer::new(translated_byte_buffer(token, buf, len))
        ) as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.inner_exclusive_access();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}