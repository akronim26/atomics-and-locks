use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release, Relaxed};

// We use MaybeUninit because initially the queue is empty and don't contain any value
pub struct OneShotChannel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    item_ready: AtomicBool,
}

// Sync -> to sync between multiple threads
// Send -> to send the data between multiple threads
unsafe impl<T> Sync for OneShotChannel<T> where T: Send {}

impl<T> OneShotChannel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            item_ready: AtomicBool::new(false),
        }
    }

    // Only call this function once
    pub unsafe fn send(&self, message: T) {
        (*self.message.get()).write(message);
        self.item_ready.store(true, Release);
    }

    pub fn is_ready(&self) -> bool {
        self.item_ready.load(Relaxed)
    }

    // Swap is an atomic operation, so there are never two threads who call receive simultaneously and see same previous value of is_ready
    pub fn receive(&self) -> T {
        if !self.item_ready.swap(false, Acquire) {
            panic!("No message ready to be received");
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }
}
