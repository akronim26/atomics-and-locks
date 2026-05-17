use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};
use std::{cell::UnsafeCell, sync::atomic::AtomicU32};

use atomic_wait::{wait, wake_all, wake_one};

pub struct Mutex<T> {
    // 0: unlocked
    // 1: locked without other threads waiting
    // 2: locked with other threads waiting
    state: AtomicU32,
    value: UnsafeCell<T>,
}

fn main() {
    println!("Hello, world!");
}

unsafe impl<T> Sync for Mutex<T> where T: Send {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        if self.state.compare_exchange(1, 2, Acquire, Relaxed).is_err() {
            while self.state.swap(2, Acquire) == 1 {
                wait(&self.state, 2);
            }
        }
        MutexGuard { mutex: self }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        if self.mutex.state.swap(0, Release) == 2 {
            wake_one(&self.mutex.state);
        }
    }
}

pub struct Condvar {
    counter: AtomicU32,
}

impl Default for Condvar {
    fn default() -> Self {
        Self {
            counter: AtomicU32::new(0),
        }
    }
}

impl Condvar {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notify_one(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_one(&self.counter);
    }

    pub fn notify_all(&self) {
        self.counter.fetch_add(1, Relaxed);
        wake_all(&self.counter);
    }

    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        let counter = self.counter.load(Relaxed);

        let mutex = guard.mutex;
        drop(guard);

        wait(&self.counter, counter);
        mutex.lock()
    }
}
