use std::collections::VecDeque;

pub struct LimitedQueue<T, const N: usize> {
    queue: VecDeque<T>,
    max_size: usize,
}

impl<T: Clone, const N: usize> LimitedQueue<T, N> {
    fn new() -> LimitedQueue<T, N> {
        LimitedQueue {
            queue: VecDeque::new(),
            max_size: N,
        }
    }

    fn push(&mut self, element: T) {
        self.queue.push_back(element);
        if self.queue.len() > self.max_size {
            self.queue.pop_front();
        }
    }

    fn push_slice(&mut self, elements: &Vec<T>) {
        self.queue.extend(elements);
        
        while self.queue.len() > self.max_size {
            self.queue.pop_front();
        }
    }
}
