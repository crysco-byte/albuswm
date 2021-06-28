use std::collections::VecDeque;
use std::mem::swap;


pub struct Stack<T> {
    before: VecDeque<T>,
    after: VecDeque<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Stack<T> {
        Stack::default()
    }

    pub fn stack_len(&self) -> usize {
        self.before.len() + self.after.len()
    }

    pub fn is_empty(&self) -> bool {
        self.before.is_empty() && self.after.is_empty()
    }

    pub fn push(&mut self, value: T) {
        self.before.extend(self.after.drain(..));
        self.after.push_front(value);
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.before.iter().chain(self.after.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.before.iter_mut().chain(self.after.iter_mut())
    }

    pub fn focused(&self) -> Option<&T> {
        self.after.get(0)
    }

    pub fn focused_mut(&mut self) -> Option<&mut T> {
        self.after.get_mut(0)
    }

    fn ensure_after_not_empty(&mut self) {
        if self.after.is_empty() && !self.before.is_empty() {
            self.after.push_front(self.before.pop_back().unwrap());
        }
    }

    pub fn remove<P>(&mut self, mut p: P) -> T
    where
        P: FnMut(&T) -> bool,
    {
        if let Some(position) = self.before.iter().position(&mut p) {
            self.before.remove(position).unwrap()
        } else {
            let position = self
                .after
                .iter()
                .position(&mut p)
                .expect("No element in stack matches predicate");
            let removed = self.after.remove(position).unwrap();
            self.ensure_after_not_empty();
            removed
        }
    }

    pub fn remove_focused(&mut self) -> Option<T> {
        let removed = self.after.pop_front();
        self.ensure_after_not_empty();
        removed
    }

    pub fn focus<P>(&mut self, mut p: P)
    where
        P: FnMut(&T) -> bool,
    {
        if let Some(position) = self.before.iter().position(&mut p) {
            for elem in self.before.drain(position..).rev() {
                self.after.push_front(elem);
            }
        } else if let Some(position) = self.after.iter().position(&mut p) {
            if position == 0 {
                return; // Already focused.
            }
            for elem in self.after.drain(..position) {
                self.before.push_back(elem);
            }
        } else {
            panic!("No element in stack matches predicate");
        }
    }

    pub fn focus_next(&mut self) {
        if self.len() < 2 {
            return;
        }
        self.before.push_back(self.after.pop_front().unwrap());
        if self.after.is_empty() {
            swap(&mut self.after, &mut self.before);
        }
    }

    pub fn focus_previous(&mut self) {
        if self.before.is_empty() {
            swap(&mut self.after, &mut self.before);
        }
        if let Some(elem) = self.before.pop_back() {
            self.after.push_front(elem);
        }
    }
}
