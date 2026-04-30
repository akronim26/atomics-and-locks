use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, fence, Ordering::{Acquire, Relaxed}};

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

    // We can get a mutable reference only when we have a single reference
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().ref_count.load(Relaxed) == 1 {
            Some(unsafe { &mut arc.ptr.as_mut().data })
        } else {
            None
        }
    }

}

// We don't implement DerefMut here because Arc is not mutable
// and hence we don't to give mutable reference
impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.data().data
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // TODO: Handle overflow
        self.data().ref_count.fetch_add(1, Relaxed);
        Arc { ptr: self.ptr }
    }
}

impl<T> Drop for Arc<T> {
    // fence ensures that the reordering does not happen when only one thread contains T
    // we have not used AcqRel here because we only want to ensure Acquire for the last thread
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Relaxed) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}
