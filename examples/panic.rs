
use std::thread;

fn main() {
    let t = thread::spawn(|| panic!("Oops - panicked again"));

    t.join().expect("Joining thread...");
}

          

        
            
// toolName: view_files
            
// status: success
          
            
// filePath: /home/guagua/rust/rust-atomics-and-locks/examples/panic.rs
          
// 这段代码演示了 Rust 中**线程 panic 的传播机制**，逐行解释如下：

// ```rust
// use std::thread;

// fn main() {
//     let t = thread::spawn(|| panic!("Oops - panicked again"));

//     t.join().expect("Joining thread...");
// }
// ```

// 1. **`use std::thread;`** — 引入标准库的线程模块。

// 2. **`thread::spawn(|| panic!("Oops - panicked again"))`** — 创建一个新线程，该线程的闭包会立即 **panic**，并附带消息 `"Oops - panicked again"`。这意味着这个子线程会崩溃。

// 3. **`t.join()`** — 主线程等待子线程 `t` 执行完毕。`join()` 返回一个 `Result<T, Box<dyn Any>>`：
//    - 如果子线程**正常完成**，返回 `Ok(返回值)`。
//    - 如果子线程 **panic**，返回 `Err(panic 信息)`。

// 4. **`.expect("Joining thread...")`** — 对 `join()` 的结果调用 `expect`。由于子线程 panic 了，`join()` 返回的是 `Err`，而 `expect` 在遇到 `Err` 时会**让主线程也 panic**，输出类似：

//    ```
//    Joining thread...: Any { .. }
//    ```

// ### 核心要点

// - Rust 中**子线程的 panic 不会自动传播到主线程**。如果主线程不调用 `join()`，子线程的 panic 会被静默忽略。
// - 只有通过 `join()` 拿到结果并主动处理（如 `expect`、`unwrap`），panic 才会"传播"到当前线程。
// - 这段代码的运行效果是：子线程 panic 打印一条消息，然后主线程因为 `expect` 也 panic，程序以非零退出码终止。

// 这来自《Rust Atomics and Locks》一书，用于说明 Rust 线程 panic 的行为——与一些语言（如 C++）中异常可能跨线程传播不同，Rust 的 panic 是**线程隔离**的。
        