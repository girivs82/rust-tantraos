//! TantraOS memory allocation implementation
//!
//! This module provides the GlobalAlloc implementation for System on TantraOS.
//! It delegates to the TantraOSAllocator which communicates with the MM tasklet.

#![allow(dead_code)]

use crate::alloc::{GlobalAlloc, Layout, System};

// TantraOS-specific allocation constants
pub const PAGE_SIZE: usize = 4096;

// TantraOS allocator that communicates with MM tasklet via IPC
pub struct TantraOSAllocator;

// Stub functions for MM tasklet communication
// TODO: Replace with actual IPC calls to MM tasklet
unsafe fn mm_request(_request: MemoryRequest) -> *mut u8 {
    // For now, return null to indicate allocation failure
    // This will be replaced with actual MM tasklet IPC
    core::ptr::null_mut()
}

enum MemoryRequest {
    Alloc { size: usize, align: usize },
    Dealloc { ptr: *mut u8, size: usize, align: usize },
    Realloc { ptr: *mut u8, old_size: usize, align: usize, new_size: usize },
}

unsafe impl GlobalAlloc for TantraOSAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe {
            mm_request(MemoryRequest::Alloc {
                size: layout.size(),
                align: layout.align(),
            })
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe {
            mm_request(MemoryRequest::Dealloc {
                ptr,
                size: layout.size(),
                align: layout.align(),
            });
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe {
            mm_request(MemoryRequest::Realloc {
                ptr,
                old_size: layout.size(),
                align: layout.align(),
                new_size,
            })
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { self.alloc(layout) };
        if !ptr.is_null() {
            unsafe {
                core::ptr::write_bytes(ptr, 0, layout.size());
            }
        }
        ptr
    }
}

// Implement GlobalAlloc for System by delegating to TantraOSAllocator
#[stable(feature = "alloc_system_type", since = "1.28.0")]
unsafe impl GlobalAlloc for System {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Special case for zero-sized allocations
        if layout.size() == 0 {
            // Use proper pointer creation for zero-sized allocations
            // This satisfies the alignment requirement without actually allocating
            return core::ptr::NonNull::<u8>::dangling().as_ptr();
        }
        unsafe { TantraOSAllocator.alloc(layout) }
    }

    #[inline]
    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        unsafe { TantraOSAllocator.alloc_zeroed(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        unsafe { TantraOSAllocator.dealloc(ptr, layout) }
    }

    #[inline]
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        unsafe { TantraOSAllocator.realloc(ptr, layout, new_size) }
    }
}