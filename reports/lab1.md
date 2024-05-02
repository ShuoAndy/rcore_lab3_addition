# 操作系统LAB1 

计12 王嘉硕

### 实现功能

LAB1简单地实现了一个用于获取当前任务的信息的系统调用 `sys_task_info` 。



 由于查询的是当前任务的状态，因此 TaskStatus 一定是 Running。而对 time 而言，我们在 struct 中维护其第一次被调度的时刻。因此我们唯一需要维护的属性就只有 syscall_times，我们需要在`syscall`中对该任务的 syscall_times 进行一次加一即可。



其余设计不再赘叙，最终在`get_current_task_info`中我们获取 TaskStatus 、syscall_times并用`get_time_ms()`减去 time 即可。

### 简答题

1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容，描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

   报错结果如下

   ```
   [ERROR][kernell .bss [x80264000，0x8028d000)
   [kernel]PageFault in application, bad addr = 0x0, bad
   instruction =0x804003ac,kernel killed it.
   [kernellIllegalInstruction in application, kernelkilled it.
   [kernellIllegalInstruction in application, kernelkilled it.
   ```

   使用的 sbi 及其版本为 `RustSBl version 0.3.0-alpha.2, adapting to RlSC-VSBI v1.0.0` 。

2. 深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code-2024S/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

   1. L40：刚进入 `__restore` 时，`a0` 代表了什么值。请指出 `__restore` 的两种使用情景。

      在进入 `__restore` 时，`a0` 寄存器被设置为当前的栈指针 `sp` 的值，其指向了 `TrapContext` 结构体。

      `__restore` 既可以恢复异常处理后的上下文，又可以从内核空间返回到用户空间。

   2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

      ```
      ld t0, 32*8(sp)
      ld t1, 33*8(sp)
      ld t2, 2*8(sp)
      csrw sstatus, t0
      csrw sepc, t1
      csrw sscratch, t2
      ```

      t0赋值给了 sstatus，它包含了当前 CPU 的状态信息。

      t1赋值了 sepc，它包含了 trap 发生前执行的最后一条指令的地址。

      t2赋值了 sscratch，它存储了用户态的栈指针 sp。

   3. L50-L56：为何跳过了 `x2` 和 `x4`？

      ```
      ld x1, 1*8(sp)
      ld x3, 3*8(sp)
      .set n, 5
      .rept 27
         LOAD_GP %n
         .set n, n+1
      .endr
      ```

      跳过 `x2` 和 `x4` 的原因是这些寄存器在保存和恢复上下文时通常不会被修改。`x2` 是栈指针（sp）寄存器，`x4` 是线程指针（tp）寄存器。在上下文切换时，栈指针和线程指针通常保持不变。

   4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      ```
      csrrw sp, sscratch, sp
      ```

      `csrrw r1, r2, r3`会将 r2 写进 r1、r3 写进 r2 ，实现交换 sp 和 sscratch 的效果。
      sp 原来是内核栈，sscratch 原来是用户栈，交换后 sp 指向用户栈， sscratch 指向内核栈。

   5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

      状态切换发生在最后一条指令 sret 上。当sret指令执行时，它会完成以下两个操作:

      - 将sstatus寄存器中的SPP(Supervisor Previous Privilege)位的值恢0复到当前的权限模式。这个位用于保存进入异常处理前的权限模式。

      - 将sepc寄存器的值设置到程序计数器(PC)，这样处理器就会继续0从发生异常或中断时的位置开始执行。

      因此，执行sret指令后，因为之前的模式是用户模式，处理器就会从内核态切换回用户态，并继续执行用户程序。

   6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

      sp 原来是用户栈，sscratch 原来是内核栈，交换后 sp 指向内核栈， sscratch 指向用户栈。

   7. 从 U 态进入 S 态是哪一条指令发生的？

      我认为应该是 ecall 指令。


### 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > 和周锦昌同学交流过简答题，代码无交流。

2. 此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

   > 除rcore实验文档外无其他参考资料。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

