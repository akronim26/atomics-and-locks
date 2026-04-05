use std::cell::UnsafeCell;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};

pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// We dont need to implement Sync because our lock will only allow one thread to access the data at a time unlike any reader-writer lock
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    // We use Acquire and Release to ensure that whatever was intended to happen during the last time it was locked has already happened
    // By making the lifetime safe, we ensure that we have the returned reference for the same lifetime as the lock
    pub fn lock<'a>(&'a self) -> &'a mut T {
        while self.locked.swap(true, Acquire) {
            // Tells the processor that we are spinning while waiting for something to change
            std::hint::spin_loop();
        }
        unsafe { &mut *self.value.get() }
    }

    // The reference to the T must be gone by the time this is called
    // Because if it is not gone and the function is called, then another thread can fetch the mutable reference and can 
    // create two mutable references at a time which is invalid
    pub unsafe fn unlock(&self) {
        self.locked.store(false, Release);
    }
}

pub fn main() {}
