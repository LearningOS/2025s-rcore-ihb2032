## 功能实现

迁移 `sys_get_time` 、 `sys_mmap` 和 `sys_munmap` ，完成 `sys_spawn` 和 `sys_set_priority`。

## 问答题

(1) 实际情况是轮到 p1 执行吗？为什么？
实际情况是p2 再次执行，而非 p1。
原因在于 8 位无符号整数的溢出。当 p2 执行后，其 stride 值增加步长（BigStride / 10）。假设 BigStride 为 255（8 位最大值），则步长为 25.5。向下取整为 25：
  - p2 原 stride = 250 → 执行后 stride = 250 + 25 = 275。
  - 275 超过 8 位无符号整数最大值（255），溢出后实际值为 275 - 256 = 19。
此时，p1.stride = 255，p2.stride = 19。调度器选择 stride 最小的进程执行，因此 p2 被选中。
溢出导致 p2 的 stride 值“回绕”，使其看似更小，从而破坏了算法的预期逻辑。
 
(2) 为什么进程优先级 ≥ 2 时，STRIDE_MAX – STRIDE_MIN ≤ BigStride / 2？
当所有进程优先级 ≥ 2 时，其步长（pass = BigStride / priority）最大为 BigStride / 2（当 priority = 2 时）。此时：
1. 调度顺序约束：调度器总是选择 stride 最小的进程执行。
2. 步长下限：每个进程执行后，其 stride 至少增加 BigStride / 2。
3. 差值控制：若某时刻 STRIDE_MAX - STRIDE_MIN > BigStride / 2，则最小 stride 的进程会被调度。执行后，其 stride 增加至少 BigStride / 2，导致：
  - 新的最小 stride ≥ 原最小 stride + BigStride / 2。
  - 若原差值超过 BigStride / 2，新的最小 stride 可能超过原最大 stride，使得差值反转或缩小。
因此，STRIDE_MAX – STRIDE_MIN 始终被控制在 BigStride / 2 以内，避免了溢出风险，同时保证了调度的公平性。

```rust
        let delta = self.0.wrapping_sub(other.0);
        if (delta as i64) < 0 {
            Some(Ordering::Less)
        } else {
            Some(Ordering::Greater)
        }
```

## 荣誉准则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
《rCore-Camp-Guide-2025S 文档》
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
