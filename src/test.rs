#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use crate::BlockingStack;
    use std::sync::Arc;
    use std::thread::JoinHandle;
    use std::{thread};

    pub const MAX_STACK_SIZE: usize = 10;
    pub const DATA: [i32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    pub const BLOCKED_DATA: [i32; 5] = [41, 42, 43, 44, 45];

    fn thread_push<T: Display + Sync>(stack: Arc<BlockingStack<'static, T>>, value: &'static T) -> JoinHandle<()> {
        thread::spawn(move || {
            stack.push(value);
            println!("+ Pushed {}", value);
        })
    }

    fn thread_pop<T: Display + Sync>(stack: Arc<BlockingStack<'static, T>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let v = stack.pop();
            println!("- Popped {}", v);
        })
    }

    #[test]
    fn test_blocked_push() {
        let stack: Arc<BlockingStack<i32>> = Arc::new(BlockingStack::new(MAX_STACK_SIZE));
        let mut push: Vec<JoinHandle<()>> = Vec::new();
        let mut blocked_push: Vec<JoinHandle<()>> = Vec::new();
        let mut pop: Vec<JoinHandle<()>> = Vec::new();

        for x in DATA.iter() {
            let s = stack.clone();
            push.push(thread_push(s, x));
        }

        push.into_iter().map(|t| t.join().unwrap()).count();

        for y in BLOCKED_DATA.iter() {
            let s = stack.clone();
            blocked_push.push(thread_push(s, y));
        }

        for _z in 0..6 {
            let s = stack.clone();
            pop.push(thread_pop(s));
        }

        pop.into_iter().map(|t| t.join().unwrap()).count();
        blocked_push.into_iter().map(|t| t.join().unwrap()).count();

        assert_eq!(9, stack.size());
    }

    #[test]
    fn test_blocked_pop() {
        let stack: Arc<BlockingStack<i32>> = Arc::new(BlockingStack::new(MAX_STACK_SIZE));
        let mut blocked_pop: Vec<JoinHandle<()>> = Vec::new();
        let mut push: Vec<JoinHandle<()>> = Vec::new();

        for _x in 0..5 {
            let s = stack.clone();
            blocked_pop.push(thread_pop(s));
        }

        for y in DATA.iter() {
            let s = stack.clone();
            push.push(thread_push(s, y));
        }

        push.into_iter().map(|t| t.join().unwrap()).count();
        blocked_pop.into_iter().map(|t| t.join().unwrap()).count();

        assert_eq!(5, stack.size());
    }
}