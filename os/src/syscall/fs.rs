//! File and filesystem-related syscalls
use crate::fs::{open_file, OpenFlags, Stat, ROOT_INODE};
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
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

/// YOUR JOB: Implement fstat.
#[allow(unused)]
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!("kernel:pid[{}] sys_fstat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let new_st = translated_refmut(token, st);

    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();

    if let Some(file) = &inner.fd_table[fd] {
        if let Some(inode_id) = &file.inode_id() {
            // info!("sys_fstat");
            // info!("{}", inode_id);
            if let Some(fstat) = Stat::scan()
                .iter_mut()
                .find(|fstat| fstat.ino == *inode_id as u64)
            {
                let mode = fstat.mode;
                let ino = fstat.ino;
                let nlink = fstat.nlink;
                // info!("{}", nlink);
                unsafe {
                    *new_st = Stat {
                        dev: 0,
                        pad: [0; 7],
                        ino,
                        mode,
                        nlink,
                    };
                }
                0
            } else {
                -1
            }
        } else {
            -1
        }
    } else {
        -1
    }
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_linkat", current_task().unwrap().pid.0);
    let token = current_user_token();
    let oldname = translated_str(token, old_name);
    let newname = translated_str(token, new_name);
    if let Some(res) = ROOT_INODE.create_link(&oldname, &newname) {
        return res as isize;
    }
    -1
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let n = translated_str(token, name);
    if let Some(inode) = ROOT_INODE.find(&n) {
        info!("Found {}", n);
        info!("{}", ROOT_INODE.ls().join(" "));
        ROOT_INODE.delete_link(&n);
        let inode_id = inode.inode_id();
        if !Stat::scan().iter_mut().any(|stat| stat.ino == inode_id as u64) {
            info!("Clear inode");
            inode.clear();
            return 0;
        }
    }
    -1
}
