use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

pub fn slice_string(s: &str, amt: usize) -> &str {
    match s.char_indices().skip(amt).next() {
        Some((pos, _)) => &s[pos..],
        None => "",
    }
}

// Thanks to https://gist.github.com/NoraCodes/e6d40782b05dc8ac40faf3a0405debd3
#[derive(Clone)]
pub struct WorkQueue<T> {
    inner: Arc<Mutex<VecDeque<T>>>,
}

impl<T> WorkQueue<T> {
    // pub fn new() -> Self {
    //     Self {
    //         inner: Arc::new(Mutex::new(VecDeque::new())),
    //     }
    // }

    pub fn from_vec(vec: Vec<T>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::from(vec))),
        }
    }

    pub fn get_work(&self) -> Option<T> {
        // Try to get a lock on the Mutex. If this fails, there is a
        // problem with the mutex - it's poisoned, meaning that a thread that
        // held the mutex lock panicked before releasing it. There is no way
        // to guarantee that all its invariants are upheld, so we need to not
        // use it in that case.
        let maybe_queue = self.inner.lock();
        // A lot is going on here. self.inner is an Arc of Mutex. Arc can deref
        // into its internal type, so we can call the methods of that inner
        // type (Mutex) without dereferencing, so this is like
        //      *(self.inner).lock()
        // but doesn't look awful. Mutex::lock() returns a
        // Result<MutexGuard<VecDeque<T>>>.

        // Unwrapping with if let, we get a MutexGuard, which is an RAII guard
        // that unlocks the Mutex when it goes out of scope.
        if let Ok(mut queue) = maybe_queue {
            // queue is a MutexGuard<VecDeque>, so this is like
            //      (*queue).pop_front()
            // Returns Some(item) or None if there are no more items.
            queue.pop_front()

            // The function has returned, so queue goes out of scope and the
            // mutex unlocks.
        } else {
            // There's a problem with the mutex.
            panic!("WorkQueue::get_work() tried to lock a poisoned mutex");
        }
    }
}
