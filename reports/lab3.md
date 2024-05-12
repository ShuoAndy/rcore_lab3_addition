# 操作系统LAB3

计12 王嘉硕

### 实现功能

LAB3在复现了前两个LAB的基础上，增添了`sys_spawn`和`sys_set_priority`系统调用。

在复现`sys_get_time`等调用时，由于ch5将任务管理和处理器管理分开实现，故而我们需要对一些`TASK_MANAGE`维护的内容移动到`PROCESSOR`中。而对LAB2的地址空间的内容无需修改。

对于`sys_spawn`，仿照`fork`和`exec`的实现，我们首先解析 elf 文件，得到其地址空间，以此新建一个 TCB。再将新的 TCB 加入当前 TCB 的 children，最后修改新的 TCB 的上下文内容。

对于`sys_set_priority`，其只需简单设置任务优先级即可。对 stride 调度算法实现于函数`stride_fetch`中。


### 简答题

1. stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

- 实际情况是轮到 p1 执行吗？为什么？
  
    实际情况不会轮到 p1 执行。因为 p2 执行完一个时间片后，p2.stride = p2.stride + pass = 250 + 10 = 260 发生上溢，8 bit无符号整型实际将 260 存储为 5。所以此时仍会执行 p2。

1. 我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。

- 为什么？尝试简单说明（不要求严格证明）。
  
    假设对某个时刻，STRIDE_MAX - STRIDE_MIN <= BigStride / 2 成立（初始状态下显然成立）：
    由于进程优先级全部 >= 2，故而 pass <= BigStride / 2。在不考虑溢出的情况下，执行 STRIDE_MIN 对应的进程后，该进程的 stride 变为 stride + pass <= BigStride / 2 + STRIDE_MIN。假若此时该进程的 stride 最大，则 STRIDE_MAX - STRIDE_MIN <= BigStride / 2。假若此时该进程的 stride 不最大，则 STRIDE_MAX - STRIDE_MIN <= BigStride / 2 仍然成立。故而由归纳法知该结论成立。

- 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。
    ```
    use core::cmp::Ordering;

    struct Stride(u64);

    impl PartialOrd for Stride {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            let big_stride: u64 = u64::MAX; 
            if self.0 < other.0 {
                if other.0 - self.0 <= big_stride / 2 { 
                    Some(Ordering::Less) 
                }
                else { 
                    Some(Ordering::Greater) 
                }
            }
            else { 
                if self.0 - other.0 <= big_stride / 2 { 
                    Some(Ordering::Greater)
                }
                else { 
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


### 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 代码方面没有和任何同学交流。

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 无。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。