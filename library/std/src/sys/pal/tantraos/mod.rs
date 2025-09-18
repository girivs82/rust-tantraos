//! Platform-specific extensions to `std` for TantraOS.
//!
//! TantraOS is an async-first operating system where everything is a tasklet.
//! This module provides the platform abstraction layer for TantraOS.

#![deny(unsafe_op_in_unsafe_fn)]

pub mod os;
pub mod time;
pub mod thread;
pub mod runtime;
pub mod panic;
pub mod rt;
pub mod process;

// Global allocator for TantraOS is implemented in sys/alloc/tantraos.rs

// Re-use implementations from unsupported until we have TantraOS-specific versions
#[path = "../unsupported/pipe.rs"]
pub mod pipe;

// Memory management

// Random number generation
pub fn fill_bytes(bytes: &mut [u8]) {
    // TODO: Use TantraOS secure random number generator
    // For now, fill with a simple pattern - this is not secure!
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
}

pub type RawOsError = i32;

use crate::io as std_io;

/// # SAFETY
/// - must be called only once during runtime initialization.
pub(crate) unsafe fn init(_argc: isize, _argv: *const *const u8, _sigpipe: u8) {
    // Initialize TantraOS runtime
    // TODO: Initialize tasklet runtime, connect to kernel services
}

/// # SAFETY
/// this is not guaranteed to run, for example when the program aborts.
/// - must be called only once during runtime cleanup.
pub unsafe fn cleanup() {
    // Cleanup TantraOS runtime
    // TODO: Cleanup tasklet runtime
}

#[inline]
pub const fn unsupported<T>() -> std_io::Result<T> {
    Err(unsupported_err())
}

#[inline]
pub const fn unsupported_err() -> std_io::Error {
    std_io::const_error!(std_io::ErrorKind::Unsupported, "operation not supported on TantraOS")
}

pub fn decode_error_kind(code: RawOsError) -> crate::io::ErrorKind {
    use crate::io::ErrorKind;

    // TantraOS error code mapping
    match code {
        // Standard POSIX-like error codes
        1 => ErrorKind::PermissionDenied,         // EPERM
        2 => ErrorKind::NotFound,                 // ENOENT
        3 => ErrorKind::NotFound,                 // ESRCH
        4 => ErrorKind::Interrupted,              // EINTR
        5 => ErrorKind::Other,                    // EIO
        6 => ErrorKind::NotFound,                 // ENXIO
        7 => ErrorKind::InvalidInput,             // E2BIG
        8 => ErrorKind::InvalidInput,             // ENOEXEC
        9 => ErrorKind::InvalidInput,             // EBADF
        10 => ErrorKind::Other,                   // ECHILD
        11 => ErrorKind::WouldBlock,              // EAGAIN
        12 => ErrorKind::OutOfMemory,             // ENOMEM
        13 => ErrorKind::PermissionDenied,        // EACCES
        14 => ErrorKind::InvalidInput,            // EFAULT
        16 => ErrorKind::ResourceBusy,            // EBUSY
        17 => ErrorKind::AlreadyExists,           // EEXIST
        18 => ErrorKind::InvalidInput,            // EXDEV
        19 => ErrorKind::NotFound,                // ENODEV
        20 => ErrorKind::NotFound,                // ENOTDIR
        21 => ErrorKind::IsADirectory,            // EISDIR
        22 => ErrorKind::InvalidInput,            // EINVAL
        23 => ErrorKind::Other,                   // ENFILE
        24 => ErrorKind::Other,                   // EMFILE
        25 => ErrorKind::Other,                   // ENOTTY
        26 => ErrorKind::FileTooLarge,            // ETXTBSY
        27 => ErrorKind::FileTooLarge,            // EFBIG
        28 => ErrorKind::StorageFull,             // ENOSPC
        29 => ErrorKind::InvalidInput,            // ESPIPE
        30 => ErrorKind::ReadOnlyFilesystem,      // EROFS
        31 => ErrorKind::Other,                   // EMLINK
        32 => ErrorKind::BrokenPipe,              // EPIPE
        _ => ErrorKind::Uncategorized,
    }
}

pub fn abort_internal() -> ! {
    // TODO: Use TantraOS kernel abort syscall
    core::intrinsics::abort();
}

pub fn is_interrupted(code: RawOsError) -> bool {
    code == 4 // EINTR
}

// Thread-local storage support
pub mod thread_local {
    pub mod key {
        // Re-exports from thread module
    }
}