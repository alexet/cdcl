/// Represents a stack of stacks, only the top stack is modifiable
/// The outer stack is never empty
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct StackStack<T> {
    values: Vec<T>,
    tops: Vec<usize>,
}

impl<T> StackStack<T> {
    /// Create a new stack of stacks
    pub fn new() -> Self {
        StackStack {
            values: Vec::new(),
            tops: Vec::new(),
        }
    }

    /// Push a new value to the top stack
    pub fn push_value(&mut self, x: T) {
        self.values.push(x);
    }

    /// Push a new stack to the stack of stacks
    pub fn push_stack(&mut self) {
        self.tops.push(self.values.len());
    }

    /// Peek the top stack
    pub fn peek_stack(&self) -> &[T] {
        let top_start: usize = self.tops.last().copied().unwrap_or(0);
        &self.values[top_start..]
    }

    /// Pop the top stack as an iterator. If it is the last stack, panics
    /// If the iterator is leaked, weird things happen.
    pub fn pop_stack_iter(&mut self) -> impl Iterator<Item = T> + '_ {
        let top_start = self.tops.pop().expect("Cannot pop the last stack");
        self.values.drain(top_start..)
    }

    pub fn len(&self) -> usize {
        self.tops.len()
    }
}
