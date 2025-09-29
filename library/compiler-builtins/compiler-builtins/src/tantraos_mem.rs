// TantraOS memory management with proper privilege separation
//
// Memory allocation/deallocation: via TypedMailbox IPC to EL1 privileged tasklet
// Memory operations within EL0 address space: direct operations (memcpy, memset, etc.)
//
// This maintains security boundaries while allowing efficient memory operations

use core::ffi::c_int;

// Memory allocation request types for TypedMailbox IPC to EL1
#[repr(C)]
#[derive(Copy, Clone)]
struct MemoryAllocationRequest {
    discriminant: u64,  // Operation type
    size: usize,        // Size to allocate
    align: usize,       // Alignment requirement
    ptr: u64,           // Pointer for deallocation
}

// Memory allocation operation types
const MALLOC_REQUEST: u64 = 0x2000;
const FREE_REQUEST: u64 = 0x2001;
const REALLOC_REQUEST: u64 = 0x2002;

// TypedMailbox IPC to EL1 privileged memory allocator service
unsafe fn memory_allocator_ipc(request: &MemoryAllocationRequest) -> *mut u8 {
    unsafe {
        // TypedMailbox uses shared memory regions for cross-privilege IPC
        let mailbox_base = 0x4000_0000 as *mut u64;

        // Write mailbox header: [target_id][from_id][msg_size]
        *mailbox_base.offset(0) = 0x1000_0002;  // target: EL1 memory allocator service

        // Get current tasklet ID from TPIDR_EL0
        let tasklet_id: u64;
        core::arch::asm!(
            "mrs {}, tpidr_el0",
            out(reg) tasklet_id,
        );
        *mailbox_base.offset(1) = tasklet_id;  // from: current tasklet ID
        *mailbox_base.offset(2) = core::mem::size_of::<MemoryAllocationRequest>() as u64;

        // Copy request to mailbox data area (starts at offset 3)
        let dest_ptr = mailbox_base.offset(3) as *mut MemoryAllocationRequest;
        core::ptr::write_volatile(dest_ptr, *request);

        // Memory barrier to ensure writes complete
        core::arch::asm!("dmb sy");

        // Set ready flag (offset 4)
        *mailbox_base.offset(4) = 1;

        // Yield to scheduler and wait for response
        loop {
            core::arch::asm!(
                "svc #0",
                in("x9") 0x12u64,  // TASKLET_YIELD
            );

            // Check if response is ready (response flag at offset 5)
            if *mailbox_base.offset(5) == 1 {
                // Read response (pointer at offset 6)
                let response = *mailbox_base.offset(6) as *mut u8;
                // Clear response flag
                *mailbox_base.offset(5) = 0;
                return response;
            }
        }
    }
}

// Direct memory operations within EL0 address space
// These are efficient and safe since they operate only within the tasklet's own memory

#[inline]
unsafe fn copy_forward(dest: *mut u8, src: *const u8, n: usize) {
    unsafe {
        let mut i = 0;
        while i < n {
            *dest.add(i) = *src.add(i);
            i += 1;
        }
    }
}

#[inline]
unsafe fn copy_backward(dest: *mut u8, src: *const u8, n: usize) {
    unsafe {
        let mut i = n;
        while i > 0 {
            i -= 1;
            *dest.add(i) = *src.add(i);
        }
    }
}

#[inline]
unsafe fn set_bytes(s: *mut u8, c: u8, n: usize) {
    unsafe {
        let mut i = 0;
        while i < n {
            *s.add(i) = c;
            i += 1;
        }
    }
}

#[inline]
unsafe fn compare_bytes(s1: *const u8, s2: *const u8, n: usize) -> c_int {
    unsafe {
        let mut i = 0;
        while i < n {
            let a = *s1.add(i);
            let b = *s2.add(i);
            if a != b {
                return (a as c_int) - (b as c_int);
            }
            i += 1;
        }
        0
    }
}

// TantraOS memory intrinsics with correct privilege separation
#[cfg(target_os = "tantraos")]
pub mod tantraos_intrinsics {
    use super::*;

    // Memory operations within EL0 address space - direct implementation
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        unsafe {
            copy_forward(dest, src, n);
        }
        dest
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memmove(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
        unsafe {
            let delta = (dest as usize).wrapping_sub(src as usize);
            if delta >= n {
                // We can copy forwards because either dest is far enough ahead of src,
                // or src is ahead of dest (and delta overflowed).
                copy_forward(dest, src, n);
            } else {
                copy_backward(dest, src, n);
            }
        }
        dest
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memset(s: *mut u8, c: c_int, n: usize) -> *mut u8 {
        unsafe {
            set_bytes(s, c as u8, n);
        }
        s
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> c_int {
        unsafe {
            compare_bytes(s1, s2, n)
        }
    }

    // Memory allocation/deallocation - via TypedMailbox IPC to EL1
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn malloc(size: usize) -> *mut u8 {
        unsafe {
            let request = MemoryAllocationRequest {
                discriminant: MALLOC_REQUEST,
                size,
                align: 8, // Default alignment
                ptr: 0,
            };
            memory_allocator_ipc(&request)
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn free(ptr: *mut u8) {
        unsafe {
            let request = MemoryAllocationRequest {
                discriminant: FREE_REQUEST,
                size: 0,
                align: 0,
                ptr: ptr as u64,
            };
            memory_allocator_ipc(&request);
        }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn realloc(ptr: *mut u8, size: usize) -> *mut u8 {
        unsafe {
            let request = MemoryAllocationRequest {
                discriminant: REALLOC_REQUEST,
                size,
                align: 8, // Default alignment
                ptr: ptr as u64,
            };
            memory_allocator_ipc(&request)
        }
    }
}