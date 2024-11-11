//! File trait & inode(dir, file, pipe, stdin, stdout)

mod inode;
mod stdio;
pub use crate::fs::inode::ROOT_INODE;
use alloc::vec::Vec;

use crate::mm::UserBuffer;

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
    /// get inode
    fn inode_id(&self) -> Option<u32>;
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
    pub pad: [u64; 7],
}

impl Stat {
    ///
    pub fn scan() -> Vec<Self> {
        let mut fstat: Vec<Self> = Vec::new();
        let info = ROOT_INODE.get_stat();
        for (_name, inode) in info {
            if let Some(f) = fstat.iter_mut().find(|fstat| fstat.ino == inode as u64) {
                // info!("scan()");
                // info!("{}", inode);
                f.nlink += 1;
            } else {
                fstat.push(Stat {
                    dev: 0,
                    ino: inode.into(),
                    mode: StatMode::FILE,
                    nlink: 1,
                    pad: [0; 7],
                })
            }
        }
        fstat
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

pub use inode::{list_apps, open_file, OSInode, OpenFlags};
pub use stdio::{Stdin, Stdout};
