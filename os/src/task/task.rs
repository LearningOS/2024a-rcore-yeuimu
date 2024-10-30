//! Types related to task management

use crate::{
    config::MAX_SYSCALL_NUM,
};

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// The time first running
    pub task_time: usize,
    /// The called syscall times and type
    pub task_syscall_times: [u32; MAX_SYSCALL_NUM]
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}
