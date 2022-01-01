use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;

use crate::fs::File;
use crate::mm::{UserBuffer, UserBufferIterator};
use crate::sync::UPSafeCell;

const MAILBOX_MAX_CAPACITY: usize = 16;

pub struct Mailbox {
    messages: UPSafeCell<Vec<String>>,
}

impl Mailbox {
    pub unsafe fn new() -> Self {
        Mailbox {
            messages: UPSafeCell::new(Vec::new()),
        }
    }
}

impl File for Mailbox {
    fn read(&self, buf: UserBuffer) -> usize {
        let mut messages = self.messages.exclusive_access();
        let s = messages.remove(0);
        drop(messages);
        let iter: UserBufferIterator = buf.into_iter();
        let mut cnt = 0;
        for (i, c) in iter.enumerate() {
            if i == s.as_bytes().len() {
                break;
            }
            unsafe {
                *c = s.as_bytes()[i];
                cnt += 1;
            }
        }
        cnt
    }

    fn write(&self, buf: UserBuffer) -> usize {
        if self.is_full() {
            return 0;
        } else {
            let s: String =
                buf.buffers.iter().map(|buffer| {
                    core::str::from_utf8(*buffer).unwrap()
                }).collect();
            let len = s.len();
            let mut messages = self.messages.exclusive_access();
            messages.push(s);
            drop(messages);
            len
        }
    }

    fn is_full(&self) -> bool {
        self.messages.exclusive_access().len() == MAILBOX_MAX_CAPACITY
    }

    fn is_empty(&self) -> bool {
        self.messages.exclusive_access().is_empty()
    }
}