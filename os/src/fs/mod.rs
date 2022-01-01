mod pipe;
mod stdio;
mod mailbox;

use crate::mm::UserBuffer;
pub trait File : Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
    fn is_full(&self) -> bool;
    fn is_empty(&self) -> bool;
}

pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
pub use mailbox::Mailbox;