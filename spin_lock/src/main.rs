use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;

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
    pub fn lock<'a>(&'a self) -> Guard<'a, T> {
        while self.locked.swap(true, Acquire) {
            // Tells the processor that we are spinning while waiting for something to change
            std::hint::spin_loop();
        }
        Guard { lock: self }
    }
}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

// Deref and DerefMut are implemented so that the lock can be used to return a reference to the data
// This allows us to use the lock in a way that is similar to a normal reference
impl<'a, T> Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<'a, T> DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.value.get() }
    }
}

// This ensures that the reference is dropped as soon as the lock is released and hence preventing any other thread from accessing the data
impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Release)
    }
}

pub fn main() {
    let x = SpinLock::new(Vec::new());
    thread::scope(|s| {
        s.spawn(|| x.lock().push(1));
        s.spawn(|| {
            let mut g = x.lock();
            g.push(2);
            g.push(2);
        });
    });
    let g = x.lock();
    assert!(g.as_slice() == [1, 2, 2] || g.as_slice() == [2, 2, 1]);
}
