## 编程作业

向 `TaskControlBlock` 添加了两个新字段：

1. `task_time`用于记录**任务第一次运行的时间**
2. `task_syscall_times`用于记录**任务系统调用次数**，下标代表系统调用编号

并且提供了三个接口：

1. `get_first_running_time` 获取其第一次调度时间
2. `save_syscall_info` 和 `get_syscall_info` 保存和设置系统调用信息

根据接口来实现`sys_task_info`，此系统调用获取当前正在执行的任务信息

## 简答题

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_\*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test-2024A/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

版本：RustSBI-QEMU Version 0.2.0-alpha.2

| 测例                       | 出错行为                     |
| ------------------------ | ------------------------ |
| ch2b_bad_address.rs      | 向非法地址0写入0                |
| ch2b_bad_instructions.rs | 在 U 态执行 S 态指令 `sret`     |
| ch2b_bad_register.rs     | 在 U 态访问 S 态寄存器 `sstatus` |

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Camp-Code-2024A/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

- L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

`a0`指向内核栈上的`TrapContex`

一个是系统调用完成后切用户态，此时`TrapContext`是`trap_handle`函数返回的

另一个是启动下一个程序，此时`TrapContext`是`run_next_app`函数构造的
	
- L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
	
```asm
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```

| CSR 名    | 该 CSR 与 Trap 相关的功能                       |
| -------- | ---------------------------------------- |
| sstatus  | `SPP` 等字段是 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息 |
| sepc     | 记录 Trap 发生之前执行的最后一条指令的地址                 |
| sscratch | 记录内核栈                                    |

- L50-L56：为何跳过了 `x2` 和 `x4`？
	
```asm
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```

 `x4` 寄存器，除非手动出于一些特殊用途使用它，否则一般也不会被用到

`x2`是`sp`，我们要基于它来找到每个寄存器应该被保存到的正确的位置

- L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

`csrrw sp, sscratch, sp`

sp指向用户栈栈顶，sscratch指向内核栈栈顶

-  `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

S 特权级的特权指令`sret`，之后 CPU 会跳转到 `sepc` 寄存器指向的那条指令，这条指令指向切换特权级之前的下一条指令

- L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

`csrrw sp, sscratch, sp`

sp指向内核栈栈顶，sscratch指向用户栈栈顶

- 从 U 态进入 S 态是哪一条指令发生的？

`eret`指令执行之后，CPU 会跳转到 `stvec` 所设置的 Trap 处理入口地址，并将当前特权级设置为 S ，然后从Trap 处理入口地址处开始执行

`stvec`在初始化时会被设置为`__alltraps`的入口

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 此外，我也参考了**以下资料**，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。