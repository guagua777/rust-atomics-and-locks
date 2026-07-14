use std::thread;

fn main() {
    let numbers = vec![1, 2, 3];


    // 使用scope
    thread::scope(|s| {
        s.spawn(|| {
            println!("length: {}", numbers.len());
        });
        s.spawn(|| {
            for n in &numbers {
                println!("{n}");
            }
        });
        
    });
}
