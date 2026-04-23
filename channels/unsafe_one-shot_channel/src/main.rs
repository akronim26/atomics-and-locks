use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release, Relaxed};
use std::thread;

// We use MaybeUninit because initially the queue is empty and don't contain any value
pub struct OneShotChannel<T> {
    message: UnsafeCell<MaybeUninit<T>>,
    in_use: AtomicBool,
    item_ready: AtomicBool,
}

// Sync -> to sync between multiple threads
// Send -> to send the data between multiple threads
unsafe impl<T> Sync for OneShotChannel<T> where T: Send {}

impl<T> OneShotChannel<T> {
    pub fn new() -> Self {
        Self {
            message: UnsafeCell::new(MaybeUninit::uninit()),
            in_use: AtomicBool::new(false),
            item_ready: AtomicBool::new(false),
        }
    }

    // Only call this function once
    pub fn send(&self, message: T) {
        if self.in_use.swap(true, Acquire) {
            panic!("Channel is already in use");
        }
        unsafe { (*self.message.get()).write(message) };
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

impl<T> Drop for OneShotChannel<T> {
    fn drop(&mut self) {
        if *self.item_ready.get_mut() {
            unsafe { (*self.message.get()).assume_init_drop() };
        }
    }
}

// The thread is parked until the other thread sends the message and the other thread unparks it
fn main() {
    let channel = OneShotChannel::new();
    let t = thread::current();
    thread::scope(|s| {
        s.spawn(|| {
            channel.send("Hello World");
            t.unpark();
        });
        while !channel.is_ready() {
            thread::park();
        }
        assert_eq!(channel.receive(), "Hello World");
    })
}