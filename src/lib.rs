use std::cmp::{Ordering};
use parking_lot::{Mutex, Condvar};
use crate::StackError::{StackFull, StackEmpty};

pub mod test;

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
            contents: Vec::new(),
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
        let mut stack = self.stack.lock();
        if stack.contents.len() >= stack.max_size {     // If full:
            self.push.wait(&mut stack);     // Wait until pushable
        }
        else {
            self.pop.notify_one();
        }
        stack.contents.push(item);
    }

    pub fn pop(&self) -> &'a T{
        let mut stack = self.stack.lock();
        if stack.contents.len() == 0 {                  // If empty
            self.pop.wait(&mut stack);      // Wait until poppable
        }
        else {
            self.push.notify_one();
        }
        stack.pop().unwrap()
    }

    pub fn size(&self) -> usize {
        let stack = self.stack.lock();
        stack.contents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    pub fn clear(&self) {
        let mut stack = self.stack.lock();
        stack.contents.clear()
    }
}