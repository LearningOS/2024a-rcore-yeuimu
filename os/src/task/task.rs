//! Types related to task management
use super::TaskContext;
use crate::config::MAX_SYSCALL_NUM;
use crate::config::TRAP_CONTEXT_BASE;
use crate::mm::VirtPageNum;
use crate::mm::{
    kernel_stack_position, MapPermission, MemorySet, PhysPageNum, VirtAddr, KERNEL_SPACE,
};
use crate::trap::{trap_handler, TrapContext};

/// The task control block (TCB) of a task.
pub struct TaskControlBlock {
    /// Save task context
    pub task_cx: TaskContext,

    /// Maintain the execution status of the current process
    pub task_status: TaskStatus,

    /// Application address space
    pub memory_set: MemorySet,

    /// The phys page number of trap context
    pub trap_cx_ppn: PhysPageNum,

    /// The size(top addr) of program which is loaded from elf file
    pub base_size: usize,

    /// Heap bottom
    pub heap_bottom: usize,

    /// Program break
    pub program_brk: usize,

    /// The time first running
    pub task_time: usize,

    /// The called syscall times and type
    pub task_syscall_times: [u32; MAX_SYSCALL_NUM],
}

impl TaskControlBlock {
    /// get the trap context
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    /// get the user token
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    /// Based on the elf info in program, build the contents of task in a new address space
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT_BASE).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        let task_control_block = Self {
            task_status,
            task_cx: TaskContext::goto_trap_return(kernel_stack_top),
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
            heap_bottom: user_sp,
            program_brk: user_sp,
            task_time: 0,
            task_syscall_times: [0; MAX_SYSCALL_NUM],
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.exclusive_access().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
    /// change the location of the program break. return None if failed.
    pub fn change_program_brk(&mut self, size: i32) -> Option<usize> {
        let old_break = self.program_brk;
        let new_brk = self.program_brk as isize + size as isize;
        if new_brk < self.heap_bottom as isize {
            return None;
        }
        let result = if size < 0 {
            self.memory_set
                .shrink_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        } else {
            self.memory_set
                .append_to(VirtAddr(self.heap_bottom), VirtAddr(new_brk as usize))
        };
        if result {
            self.program_brk = new_brk as usize;
            Some(old_break)
        } else {
            None
        }
    }
    /// malloc a memory block
    #[allow(unused)]
    pub fn mmap(&mut self, start: usize, len: usize, port: usize) -> Result<bool, &'static str> {
        let start_v = VirtAddr::from(start);
        let end_v = VirtAddr::from(VirtAddr::from(start + len).ceil());

        // 检查地址是否按页对齐
        if !start_v.aligned() || !end_v.aligned() {
            return Err("Address is not page-aligned.");
        }

        // 检查是否有重叠
        if self.memory_set.is_overlapping(start_v, end_v) {
            return Err("Memory area is overlapping.");
        }

        // 检查权限位是否有效
        if port & !0b111 != 0 {
            return Err("Invalid permission bits.");
        }

        // 必须至少有一个权限位被设置
        if port & 0b111 == 0 {
            return Err("At least one permission must be set.");
        }

        let mut perm = MapPermission::U; // 默认权限
        if port & 1 != 0 {
            perm |= MapPermission::R;
        }
        if port & (1 << 1) != 0 {
            perm |= MapPermission::W;
        }
        if port & (1 << 2) != 0 {
            perm |= MapPermission::X;
        }

        // 插入映射
        self.memory_set.insert_framed_area(start_v, end_v, perm);
        Ok(true) // 映射成功
    }

    /// dealloc a memory block
    pub fn unmap(&mut self, start: usize, len: usize) -> Result<bool, &'static str> {
        let start_v = VirtAddr::from(start);
        let end_v = VirtAddr::from(start + len);

        // 检查地址是否按页对齐
        if !start_v.aligned() || !end_v.aligned() {
            return Err("Address is not page-aligned.");
        }

        if self.memory_set.unmap_area(VirtPageNum::from(start_v), VirtPageNum::from(end_v)) {
            Ok(true)
        } else {
            Err("没有映射这个地址")
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
/// task status: UnInit, Ready, Running, Exited
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
