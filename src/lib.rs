use std::cmp::{Ordering};
use crate::StackError::{StackFull, StackEmpty};
use std::sync::{Mutex, Condvar};

#[derive(thiserror::Error, Debug)]
pub enum StackError {
    #[error("stack is empty")]
    StackEmpty,
    #[error("stack is full")]
    StackFull
}

pub struct Stack<'a, T> {
    contents: Vec<&'a T>,
    max_size: usize
}

impl<'a, T> Stack<'a, T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            contents: Vec::with_capacity(max_size),
            max_size
        }
    }

    pub fn push(&mut self, item: &'a T) -> Result<(), StackError> {
        match self.contents.len().cmp(&self.max_size) {
            Ordering::Less => {
                self.contents.push(item);
                Ok(())
            }
            _ => {
                Err(StackFull)
            }
        }
    }

    pub fn pop(&mut self) -> Result<&'a T, StackError> {
        match self.contents.len() {
            0 => {
                Err(StackEmpty)
            }
            _ => {
                Ok(self.contents.pop().unwrap())
            }
        }
    }

    pub fn size(&self) -> usize {
        self.contents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn clear(&mut self) {
        self.contents.clear()
    }
}


pub struct BlockingStack<'a, T> {
    stack: Mutex<Stack<'a, T>>,
    push: Condvar,
    pop: Condvar
}

impl<'a, T> BlockingStack<'a, T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            stack: Mutex::new(Stack::new(max_size)),
            push: Condvar::new(),
            pop: Condvar::new(),
        }
    }

    pub fn push(&self, item: &'a T) {
        let mut stack = self.push.wait_while(
            self.stack.lock().unwrap(),
            |s| s.contents.len() >= s.max_size
        ).unwrap();
        stack.push(item).unwrap();
        self.pop.notify_one();
    }

    pub fn pop(&self) -> &'a T {
        let mut stack = self.pop.wait_while(
            self.stack.lock().unwrap(),
            |s| s.is_empty()
        ).unwrap();
        self.push.notify_one();
        stack.pop().unwrap()
    }

    pub fn size(&self) -> usize {
        let stack = self.stack.lock().unwrap();
        stack.contents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn clear(&self) {
        let mut stack = self.stack.lock().unwrap();
        self.push.notify_all();
        stack.contents.clear()
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Display;
    use std::sync::Arc;
    use std::thread::JoinHandle;
    use std::{thread};
    use crate::BlockingStack;

    pub const MAX_STACK_SIZE: usize = 10;
    pub const DATA: [i32; 10] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    pub const BLOCKED_DATA: [i32; 5] = [41, 42, 43, 44, 45];

    fn thread_push<T: Display + Sync>(stack: Arc<BlockingStack<'static, T>>, value: &'static T) -> JoinHandle<()> {
        thread::spawn(move || {
            stack.push(value);
        })
    }

    fn thread_pop<T: Display + Sync>(stack: Arc<BlockingStack<'static, T>>) -> JoinHandle<()> {
        thread::spawn(move || {
            stack.pop();
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

        push.into_iter()
            .map(|t| t.join().unwrap())
            .count();

        for y in BLOCKED_DATA.iter() {
            let s = stack.clone();
            blocked_push.push(thread_push(s, y));
        }

        for _z in 0..6 {
            let s = stack.clone();
            pop.push(thread_pop(s));
        }

        pop.into_iter()
            .map(|t| t.join().unwrap())
            .count();

        blocked_push.into_iter()
            .map(|t| t.join().unwrap())
            .count();

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

        push.into_iter()
            .map(|t| t.join().unwrap())
            .count();

        blocked_pop.into_iter()
            .map(|t| t.join().unwrap())
            .count();

        assert_eq!(5, stack.size());
    }

    #[test]
    fn push_only() {
        let stack: Arc<BlockingStack<i32>> = Arc::new(BlockingStack::new(MAX_STACK_SIZE));
        let mut push: Vec<JoinHandle<()>> = Vec::new();
        for x in DATA.iter() {
            let s = stack.clone();
            push.push(thread_push(s, x));
        }
        push.into_iter()
            .map(|t| t.join().unwrap())
            .count();
    }

    #[test]
    fn push_pop_full() {
        let stack: Arc<BlockingStack<i32>> = Arc::new(BlockingStack::new(MAX_STACK_SIZE));
        let mut push: Vec<JoinHandle<()>> = Vec::new();
        let mut pop: Vec<JoinHandle<()>> = Vec::new();

        for x in DATA.iter() {
            let s = stack.clone();
            push.push(thread_push(s, x));
        }

        for _y in 0..MAX_STACK_SIZE {
            let s = stack.clone();
            pop.push(thread_pop(s));
        }

        pop.into_iter()
            .map(|t| t.join().unwrap())
            .count();

        push.into_iter()
            .map(|t| t.join().unwrap())
            .count();
    }
}
