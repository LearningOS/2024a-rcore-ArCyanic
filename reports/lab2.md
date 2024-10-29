# 实现的功能
1. 将`TaskInfo`结构从`process.rs`移动到`task.rs`，并修改相关`mod`语句。
2. 修改`TaskManagerInfo`，增加了`first_scheduled_times`(第一次任务调度时间数组)和`syscall_times`(任务系统调用次数数组)。
3. 在每次调度任务时，判断当前是否是第一次调度，如果是就更新`first_scheduled_times`。
4. 在`syscall`函数内加入`update_syscall_times()`函数，这样在每次进行函数调用时，更新`syscall_times`数组。
5. 在`sys_task_info()`函数内直接修改传入的`TaskInfo`指针的指向地址的内容。

# 简答题
1. 使用`RustSBI 0.3.0-alpha.2`，得到相关报错信息:
  ``` text
  [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
  [kernel] IllegalInstruction in application, kernel killed it.
  ```

2. 深入理解`Trap.S`:

  1.`__restore`的两种使用场景：
    - 当`__switch()`被调用时，该函数首先会保存当前上下文信息，然后会恢复下一个任务的上下文信息，其中就包括`ra`的内容，而寄存器`a0`的内容则不会发生变化，仍然是指向当前`TaskContext`的指针。寄存器`ra`的内容被设置为`__restore`函数的入口地址，`__switch()`返回时，由于`ra`设置为`__restore()`的入口地址，因此程序接下去会执行`__restore()`的内容。此时`a0`的值仍然是当前指向任务上下文的指针。
    - 当`trap`被触发时，会首先调用`__alltraps()`，`__alltraps()`的结尾会调用`trap_handler()`，`trap_handler()`返回后紧接着执行的就是`__restore()`。而在RISC-V中，函数返回值被保存在`a0`寄存器中，因此当`__restore()`执行时，寄存器`a0`的值就是`trap_handler()`的返回值，也就是指向当前`TrapContext`的指针。

  2. L43-L48处理了`sstatus, sepc, sscratch`三个寄存器。
    - `sstatus`: 该寄存器保存了各种状态位，包括是否使能中断（sie），Trap发生前CPU处于哪个特权级（SPP）等信息。如果SPP的值为U,则sret后操作系统会进入用户态。
    - `sepc`: 改寄存器记录了Trap前最后一条指令的地址，如果Trap前是用户态，则sret后程序会从`sepc`所指向的地址开始执行。
    - `sscratch`: 通常作为内核在处理Trap时的临时存储寄存器。进入内核态时，其保存了内核栈地址，后续可用该寄存器保存Trap上下文。后续处理时，其可用来保存用户栈地址，帮助恢复Trap前的上下文。 

  3. 首先`x4`也叫`tp`，即`thread pointer`，在这些应用程序中没有使用到。`x2`是`sp`，该段代码需要`sp`来帮助恢复上下文，在`__restore`的最后才会通过`csrrw sp, sscratch, sp`恢复。

  4. 此时`sp`指向用户栈，`sscratch`指向内核栈。

  5. `__restore`中发生状态切换的指令是`sret`，该指令会使程序返回到`sepc`所指向的地址开始执行，也就是返回到用户态的代码，同时会修改相关寄存器，如`sstatus`的值。

  6. 此时`sp`指向内核栈，`sscratch`指向用户栈。

  7. 执行`ecall`后从U态进入S态。

# 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

  无，完全独立完成实验。

2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：

  无，只看了实验文档，从未在网络上寻找其他相关实现参考。

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
