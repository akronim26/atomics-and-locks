use std::cell::UnsafeCell;
use std::ops::Deref;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, fence, Ordering::{Acquire, Relaxed}};

struct ArcData<T> {
    // Number of `Arc`s
    data_ref_count: AtomicUsize,
    // Number of `Arc`s and `Weak`s
    alloc_ref_count: AtomicUsize,
    // The actual data is optional because it may be only held by `Weak`
    data: UnsafeCell<Option<T>>,
}

pub struct Arc<T> {
    weak: Weak<T>
}

// A NonNull pointer is a pointer which cannot be null
pub struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

// Sync is required to maintain T in sync with all the threads
// Send is required for the last thread to actually drop T
unsafe impl<T: Send + Sync> Send for Weak<T> {}
unsafe impl<T: Send + Sync> Sync for Weak<T> {}

impl<T> Arc<T> {
    // Box::leak prevents automatic dropping of the data
    // This is done because Arc wants manual control
    pub fn new(data: T) -> Self {
        Arc {
            weak: Weak {
                ptr: NonNull::from({
                    Box::leak(Box::new(ArcData{
                        data_ref_count: AtomicUsize::new(1),
                        alloc_ref_count: AtomicUsize::new(1),
                        data: UnsafeCell::new(Some(data))
                    }))
                })
            }
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.weak.ptr.as_ref() }
    }

    // We can get a mutable reference only when we have a single reference
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc.data().data_ref_count.load(Relaxed) == 1 {
            Some(unsafe { arc.weak.ptr.as_mut().data.get_mut().as_mut().unwrap() })
        } else {
            None
        }
    }

    pub fn downgrade(&self) -> Weak<T> {
        self.weak.clone()
    }

}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe {
            self.ptr.as_ref()
        }
    }

    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut n = self.data().data_ref_count.load(Relaxed);
        // We loop here because if the process failed due to a race condition, then it can be 
        // re-executed. 
        loop {
            if n == 0 {
                return None;
            }
            // The `compare_exchange_weak` ensures that if the current value of `data_ref_count`
            // is still `n` then change it to `n + 1` as a `Weak` pointer is being converted to 
            // `Arc`
            if let Err(e) = self.data().data_ref_count.compare_exchange_weak(n, n + 1, Relaxed, Relaxed) {
                // If another thread simultaneously upgraded another `Weak` pointer due to which this 
                // lead to an error here, then e is the new value of the n
                n = e;
                continue;
            }
            return Some(Arc{
                weak: self.clone()
            })
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.data().alloc_ref_count.fetch_add(1, Relaxed);
        Weak {
            ptr: self.ptr
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().alloc_ref_count.fetch_sub(1, Relaxed) == 1 {
            fence(Acquire);
            unsafe {
                drop(Box::from_raw(self.ptr.as_ptr()));
            }
        }
    }
}

// We don't implement DerefMut here because Arc is not mutable
// and hence we don't to give mutable reference
impl<T> Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {
            &(*self.weak.data().data.get()).as_ref().unwrap()
        }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        // TODO: Handle overflow
        let weak = self.weak.clone();
        self.data().data_ref_count.fetch_add(1, Relaxed);
        Arc { weak }
    }
}

impl<T> Drop for Arc<T> {
    // fence ensures that the reordering does not happen when only one thread contains T
    // we have not used AcqRel here because we only want to ensure Acquire for the last thread
    fn drop(&mut self) {
        if self.data().data_ref_count.fetch_sub(1, Relaxed) == 1 {
            fence(Acquire);
            unsafe {
                *self.weak.data().data.get() = None
            }
        }
    }
}
