use std::sync::Arc;
use crate::blockingstack::BlockingStack;
use std::thread;
use std::thread::JoinHandle;

pub mod blockingstack;

pub const MAX_STACK_SIZE: usize = 20;
pub const DATA: [i32; 20] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20];


fn main() {
    let stack: Arc<BlockingStack<i32>> = Arc::new(BlockingStack::new(MAX_STACK_SIZE));
    let mut threads: Vec<JoinHandle<()>> = Vec::new();

    for x in DATA.iter() {
        let s = stack.clone();
        threads.push(thread::spawn(move || {
            s.push(x)
        }));
    }

    for thread in threads {
        thread.join().unwrap();
    }

    println!("{}", stack.size())
}
