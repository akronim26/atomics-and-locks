use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>
}

unsafe impl<T> Sync for SpinLock<T> where T: Send {}


impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    // We use Acquire and Release to ensure that whatever was intended to happen during the last time it was locked has already happened
    pub fn lock(&self) {
        while self.locked.swap(true, Acquire) {
            // Tells the processor that we are spinning while waiting for something to change
            std::hint::spin_loop();
        }
    }

    pub fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

pub fn main() {

}