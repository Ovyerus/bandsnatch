use phf::phf_map;
use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    sync::{Arc, Mutex},
};

// From https://github.com/Ezwen/bandcamp-collection-downloader/blob/master/src/main/kotlin/bandcampcollectiondownloader/core/Constants.kt#L7
static REPLACEMENT_CHARS: phf::Map<&str, &str> = phf_map! {
    ":" => "꞉",
    "/" => "／",
    "\\" => "⧹",
    "\"" => "＂",
    "*" => "⋆",
    "<" => "＜",
    ">" => "＞",
    "?" => "？",
    "|" => "∣"
};

// NTFS doesn't like these and pretty much shits itself if you try to do
// anything to files/folders containing em.
static UNSAFE_NTFS_ENDINGS: &[char] = &['.', ' '];

pub fn make_string_fs_safe(s: &str) -> String {
    let mut str = s.to_string();

    for (from, to) in REPLACEMENT_CHARS.entries() {
        str = str.replace(from, to);
    }

    if UNSAFE_NTFS_ENDINGS.contains(&str.chars().last().unwrap()) {
        str.push('_');
    }

    str
}

pub fn slice_string(s: &str, amt: usize) -> &str {
    match s.char_indices().nth(amt) {
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

const DEFAULT_BUF_SIZE: usize = 8192;

// `std::io::copy` slightly modified to update a progress bar as it copies
// https://doc.rust-lang.org/1.8.0/src/std/up/src/libstd/io/util.rs.html#46-61
pub fn copy_with_progress<R: ?Sized, W: ?Sized>(
    reader: &mut R,
    writer: &mut W,
    pb: &indicatif::ProgressBar,
) -> io::Result<u64>
where
    R: Read,
    W: Write,
{
    let mut buf = [0; DEFAULT_BUF_SIZE];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
        pb.set_position(written);
    }
}
