use std::sync::Mutex;

pub struct Channel<T> {
    queue: Mutex<VecDeque<T>>,
    item_ready: Condvar
}

impl<T> Channel<T> {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            item_ready: Condvar::new()
        }
    }
}