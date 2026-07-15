use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;

// This version is problematic.
fn allocate_new_id() -> u32 {
    static NEXT_ID: AtomicU32 = AtomicU32::new(0);
    let id = NEXT_ID.fetch_add(1, Relaxed);
    assert!(id < 1000, "too many IDs!");
    id
}

fn main() {
    dbg!(allocate_new_id()); // This will produce a zero.

    for _ in 1..1000 {
        allocate_new_id(); // 1 through 999.
    }

    println!("overflowing the counter... (this might take a few hours)");

    std::panic::set_hook(Box::new(|_| {}));

    for _ in 1000..=u32::MAX {
        let _ = std::panic::catch_unwind(|| allocate_new_id());
    }

    println!("overflowed!");

    dbg!(allocate_new_id()); // ⚠️ This will produce zero again. ⚠️
}


// 让我先确认当前文件的内容，然后给你详细解释。


// 好的，让我给你逐段详细解释这份代码。这份代码是用来演示 `fetch_add` + `assert` 方案**仍然存在的问题**。

// ---

// ## `ch2-09-id-allocation-panic.rs` 完整解析

// ### 代码结构总览

// ```
// ┌─────────────────────────────────────────┐
// │  1-2: 导入依赖                           │
// │  5-10: allocate_new_id() 函数（核心）    │
// │  12-30: main() 测试函数                  │
// └─────────────────────────────────────────┘
// ```

// ---

// ### 第一部分：`allocate_new_id()` 函数（第 5-10 行）

// ```rust
// // This version is problematic.
// fn allocate_new_id() -> u32 {
//     static NEXT_ID: AtomicU32 = AtomicU32::new(0);
//     let id = NEXT_ID.fetch_add(1, Relaxed);
//     assert!(id < 1000, "too many IDs!");
//     id
// }
// ```

// #### 执行流程拆解

// | 步骤 | 代码 | 说明 |
// |------|------|------|
// | 1 | `static NEXT_ID: ...` | 静态变量，程序启动时初始化为 0，全局唯一 |
// | 2 | `let id = NEXT_ID.fetch_add(1, Relaxed)` | **先加 1**，返回**加之前的值** |
// | 3 | `assert!(id < 1000, ...)` | **检查**：如果 `id >= 1000`，触发 panic |
// | 4 | `id` | 返回分配的 ID |

// #### ⚠️ 关键问题：**"先加后查"**

// ```
// 调用顺序：
//     fetch_add(1) → 计数器已经 +1 了（不管是否超过限制）
//     assert!(id < 1000) → 检查是否超限

// 假设计数器当前是 999：
//     fetch_add(1) → 返回 999，NEXT_ID 变成 1000
//     assert!(999 < 1000) → ✅ 通过

// 下一次调用，计数器是 1000：
//     fetch_add(1) → 返回 1000，NEXT_ID 变成 1001
//     assert!(1000 < 1000) → ❌ panic!

//     ⚠️ 问题：NEXT_ID 已经被改成 1001 了！即使 panic，计数器也不会回退。
// ```

// **panic 发生时**：函数不会返回值，但计数器 `NEXT_ID` **已经增加了**。这是这段代码"有问题"的核心原因。

// ---

// ### 第二部分：`main()` 函数（第 12-30 行）

// #### 阶段一：正常分配（第 13-17 行）

// ```rust
// dbg!(allocate_new_id());   // 返回 0

// for _ in 1..1000 {
//     allocate_new_id();     // 分配 1, 2, 3, ..., 999
// }
// ```

// **结果**：`NEXT_ID` 现在的值是 **1000**，已分配 ID 0~999。

// #### 阶段二：静音 panic（第 19-21 行）

// ```rust
// println!("overflowing the counter... (this might take a few hours)");

// std::panic::set_hook(Box::new(|_| {}));  // 把 panic 的默认输出替换成空操作
// ```

// **目的**：后面的循环每次调用都会 panic，如果不静音，终端会被 panic 输出淹没。

// #### 阶段三：触发大量 panic（第 23-25 行）

// ```rust
// for _ in 1000..=u32::MAX {
//     let _ = std::panic::catch_unwind(|| allocate_new_id());
// }
// ```

// 这是最关键的部分，让我仔细解释：

// ##### `std::panic::catch_unwind`

// ```rust
// pub fn catch_unwind<F: FnOnce() -> R + UnwindSafe, R>(f: F) -> Result<R, Box<dyn Any + Send>>
// ```

// - **作用**：执行闭包，如果闭包内发生 panic，**捕获它而不是让程序崩溃**
// - **返回值**：
//   - `Ok(result)`：闭包正常执行，返回结果
//   - `Err(payload)`：闭包内发生了 panic，payload 是 panic 的负载

// ##### 循环内发生了什么？

// ```
// 当前 NEXT_ID = 1000

// 第 1000 次循环调用:
//     fetch_add(1) → 返回 1000, NEXT_ID = 1001
//     assert!(1000 < 1000) → ❌ panic!
//     catch_unwind 捕获 panic → 返回 Err，被忽略

// 第 1001 次循环调用:
//     fetch_add(1) → 返回 1001, NEXT_ID = 1002
//     assert!(1001 < 1000) → ❌ panic!
//     catch_unwind 捕获 panic → 返回 Err，被忽略

// 第 1002 次循环调用:
//     fetch_add(1) → 返回 1002, NEXT_ID = 1003
//     ...

// ... (重复 4,294,966,295 次) ...

// 最后几次:
//     fetch_add(1) → 返回 4294967294, NEXT_ID = 4294967295
//     assert!(4294967294 < 1000) → ❌ panic!

//     fetch_add(1) → 返回 4294967295, NEXT_ID = 0  ← ⚠️ 溢出回绕！
//     assert!(4294967295 < 1000) → ❌ panic!
// ```

// **重点观察**：
// - 每次调用 `fetch_add` 都成功增加了计数器（即使随后 panic）
// - panic 被 `catch_unwind` 捕获，程序继续运行
// - 计数器不断增长，直到 `u32::MAX` 后**溢出回绕到 0**

// #### 阶段四：演示最终问题（第 27-29 行）

// ```rust
// println!("overflowed!");

// dbg!(allocate_new_id());  // ⚠️ 这会再次返回 0！
// ```

// 经过前面的循环，计数器已经溢出回绕。现在：

// ```
// NEXT_ID 当前值 = 0（因为溢出回绕了）

// 调用 allocate_new_id():
//     fetch_add(1) → 返回 0，NEXT_ID = 1
//     assert!(0 < 1000) → ✅ 通过！
//     return 0
// ```

// **ID 0 被重复分配了！** 🚨

// ---

// ### 这份代码演示的完整问题链

// ```
// ┌──────────────────────────────────────────────────────┐
// │ 问题 1: fetch_add 先执行，后检查                         │
// │   → 即使 assert 失败，计数器也已经被修改                  │
// ├──────────────────────────────────────────────────────┤
// │ 问题 2: panic 被 catch_unwind 捕获，程序继续运行          │
// │   → 计数器虽然一直在 panic，但还在不断 +1                  │
// ├──────────────────────────────────────────────────────┤
// │ 问题 3: u32 溢出后静默回绕                               │
// │   → NEXT_ID 从 u32::MAX "悄悄" 变回 0                   │
// ├──────────────────────────────────────────────────────┤
// │ 最终结果: ID 重复分配                                   │
// │   → 原本应该唯一的 ID 0 被再次分配                        │
// └──────────────────────────────────────────────────────┘
// ```

// ### 对比修复方案（ch2-10 和 ch2-12）

// | 版本 | 策略 | 问题是否存在 |
// |------|------|-----------|
// | ch2-08 | `fetch_add` 无任何检查 | ✅ 溢出回绕 |
// | ch2-09 | `fetch_add` + `assert`（先加后查） | ✅ panic 时计数器已增长，仍会溢出 |
// | **ch2-10** | `fetch_add` + `fetch_sub` 回退 | ⚠️ 部分修复（超限会回退，但 fetch_sub 本身也有竞争问题） |
// | **ch2-12** | `compare_exchange` 循环（先查后改） | ✅ 正确修复 |

// ### 一句话总结

// > 这份代码演示：**先用 `fetch_add` 增加计数器，再用 `assert` 检查是否超限**是不够的——即使 panic，计数器已经被修改了。配合 `catch_unwind` 捕获 panic 让程序继续运行，最终计数器会溢出回绕，导致 **ID 被重复分配**，这是一个隐蔽的正确性 bug。