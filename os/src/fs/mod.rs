//! File trait & inode(dir, file, pipe, stdin, stdout)

mod inode;
mod stdio;
use crate::fs::inode::ROOT_INODE;
use alloc::vec::Vec;
use lazy_static::*;

use crate::mm::UserBuffer;
use crate::sync::UPSafeCell;
use alloc::sync::Arc;

/// trait File for all file types
pub trait File: Send + Sync {
    /// the file readable?
    fn readable(&self) -> bool;
    /// the file writable?
    fn writable(&self) -> bool;
    /// read from the file to buf, return the number of bytes read
    fn read(&self, buf: UserBuffer) -> usize;
    /// write to the file from buf, return the number of bytes written
    fn write(&self, buf: UserBuffer) -> usize;
}

/// The stat of a inode
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Stat {
    /// ID of device containing file
    pub dev: u64,
    /// inode number
    pub ino: u64,
    /// file type and mode
    pub mode: StatMode,
    /// number of hard links
    pub nlink: u32,
    /// unused pad
    pad: [u64; 7],
}

impl Stat {
    pub fn scan() -> Vec<Self> {
        let mut stats: Vec<Self> = Vec::new();
        let info = ROOT_INODE.get_stat();
        for (name, inode) in info {
            if let Some(stat) = stats
                .iter_mut()
                .find(|stat: &&mut Stat| stat.ino == inode.into())
            {
                stat.nlink += 1;
            } else {
                stats.push(Stat {
                    dev: 0,
                    ino: inode.into(),
                    mode: StatMode::DIR,
                    nlink: 1,
                    pad: [0; 7],
                })
            }
        }
        stats
    }
}

bitflags! {
    /// The mode of a inode
    /// whether a directory or a file
    pub struct StatMode: u32 {
        /// null
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}

lazy_static! {
    pub static ref STATS: Arc<UPSafeCell<Vec<Stat>>> =
        Arc::new(unsafe { UPSafeCell::new(Stat::scan()) });
}

pub use inode::{list_apps, open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};
