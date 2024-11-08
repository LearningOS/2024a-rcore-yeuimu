## 编程作业

`spawn` 系统调用是 `fork` 和 `exec` 系统调用的改良版，不需要复制父进程的内存空间和其他东西，而是直接像 `TaskControlBlock::new` 那样创建一个全新的内存空间，然后创建其执行环境，不同的是 `spawn` 创建的是子进程

`TaskManager` 的 `fetch` 方法中实现了 **stride 调度算法**，每次调度新任务时，选取**步长总量最小的**任务，之后增加此任务的步长，**步长增量**与任务优先级成正比，如此循环往复

`set_priority` 系统调用则可以改变优先级

## 简答题

如果 `p1.stride = 255`，`p2.stride = 250`，p2 继续执行后其 stride 将溢出回到 0。因此，溢出会导致进程顺序错误，使得实际调度与期望不符。

在不溢出时，当进程优先级>=2时，最大的pass是 BigStride/2，最小的pass是 BigStride/16，它们之间差值就是 BigStride/2 - BigStride/16，所以它们的差值肯定小于 BigStride/2，则`STRIDE_MAX – STRIDE_MIN <= BigStride / 2`

如果出现不符合这条公式的情况，那就是溢出了，我们可以反转对比的结果

于是：

```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let big_stride = u64::MAX;
        let half_stride = big_stride / 2;
        
        // 计算两者的差值并取绝对值
        let diff = if self.0 > other.0 {
            self.0 - other.0
        } else {
            other.0 - self.0
        };

        // 如果差值小于 BigStride / 2，直接比较
        if diff < half_stride {
            self.0.partial_cmp(&other.0)
        } else {
            // 如果差值大于等于 BigStride / 2，反转比较结果
            if self.0 < other.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```

上面是以八位的stride为例，这里实现用的是64位的stride，所以这里的stride取64位最大值

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与**以下各位**就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

    无

2. 此外，我也参考了**以下资料**，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

    无

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
