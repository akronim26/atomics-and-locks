use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::AtomicUsize;

struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

// A NonNull pointer is a pointer which cannot be null
pub struct Arc<T> {
    ptr: NonNull<ArcData<T>>,
}

// Sync is required to maintain T in sync with all the threads
// Send is required for the last thread to actually drop T
unsafe impl<T: Send + Sync> Send for Arc<T> {}
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    // Box::leak prevents automatic dropping of the data
    // This is done because Arc wants manual control
    pub fn new(data: T) -> Self {
        Arc {
            ptr: NonNull::from(Box::leak(Box::new(ArcData {
                ref_count: AtomicUsize::new(1),
                data,
            }))),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}
