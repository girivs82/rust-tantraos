//! TantraOS thread implementation (tasklets)

#![allow(dead_code)]

use crate::io;
use crate::time::Duration;

// TantraOS thread handle (representing a tasklet)
pub struct Thread {
    id: u32,
}

impl Thread {
    pub fn id(&self) -> u32 {
        self.id
    }
}

// Thread-local key implementation
pub mod key {
    use crate::ptr;

    // Simple key implementation - for now just stub
    pub struct Key {
        key: u32,
    }

    impl Key {
        pub const fn new() -> Key {
            Key { key: 0 }
        }

        pub unsafe fn set(&self, _value: *mut u8) {
            // TODO: Implement TLS set
        }

        pub unsafe fn get(&self) -> *mut u8 {
            // TODO: Implement TLS get
            ptr::null_mut()
        }

        pub unsafe fn destroy(&self) {
            // TODO: Implement TLS destroy
        }
    }

    pub struct LazyKey {
        key: Key,
    }

    impl LazyKey {
        pub const fn new() -> LazyKey {
            LazyKey { key: Key::new() }
        }

        pub unsafe fn get(&self) -> *mut u8 {
            unsafe { self.key.get() }
        }
    }

    // Global functions required by std
    pub unsafe fn set(key: &Key, value: *mut u8) {
        unsafe { key.set(value); }
    }

    pub unsafe fn get(key: &Key) -> *mut u8 {
        unsafe { key.get() }
    }
}

pub fn available_parallelism() -> io::Result<usize> {
    // TODO: Get actual CPU count from TantraOS
    Ok(1)
}

pub fn current() -> Thread {
    Thread {
        // TODO: Get actual tasklet ID
        id: 1
    }
}

pub fn yield_now() {
    // TODO: Yield to TantraOS scheduler
}

pub fn sleep(dur: Duration) {
    let _ = dur;
    // TODO: Sleep via TantraOS scheduler
}

pub fn park() {
    // TODO: Park current tasklet
}

pub fn park_timeout(dur: Duration) {
    let _ = dur;
    // TODO: Park with timeout
}

pub fn unpark(thread: &Thread) {
    let _ = thread;
    // TODO: Unpark specific tasklet
}

// Required for Thread Sanitizer
pub fn exit() {
    // TODO: Exit current tasklet
}