//! TantraOS async runtime interface
//!
//! This module provides the interface to TantraOS's kernel async runtime
//! via TypedMailbox with automatic cross-privilege support. This aligns
//! with TantraOS's design philosophy where everything communicates via
//! message passing.

#![allow(dead_code)]

use core::future::Future;
use core::ptr::addr_of_mut;

/// Well-known mailbox IDs for runtime services
#[allow(dead_code)]
pub mod mailbox_ids {
    /// Runtime service mailbox - kernel provides this
    pub const RUNTIME_SERVICE: u64 = 0x1000;

    /// Per-tasklet waker notification mailbox
    pub const WAKER_BASE: u64 = 0x2000;
}

/// Messages to the kernel runtime service
/// These are sent via TypedMailbox<RuntimeRequest>
#[repr(C)]
#[derive(Clone)]
#[allow(dead_code)]
pub enum RuntimeRequest {
    /// Register a new async task
    RegisterTask {
        task_id: u64,
        entry_point: usize,
    },
    /// Yield execution to scheduler
    Yield,
    /// Block on a future until completion
    BlockOn {
        future_ptr: usize,
    },
    /// Exit the current tasklet
    Exit {
        status: i32,
    },
}

/// Responses from the kernel runtime
/// Received via TypedMailbox<RuntimeResponse>
#[repr(C)]
#[derive(Clone)]
#[allow(dead_code)]
pub enum RuntimeResponse {
    /// Task registered successfully
    TaskRegistered {
        task_id: u64,
    },
    /// Future completed with result
    Completed,
    /// Waker notification
    Wake {
        task_id: u64,
    },
    /// Error response
    Error {
        code: u32,
    },
}

/// Stub for TypedMailbox - will be provided by TantraOS std
/// This automatically handles cross-privilege communication
#[allow(dead_code)]
pub struct TypedMailbox<T> where T: Send + Clone + 'static {
    channel_id: u64,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> TypedMailbox<T> where T: Send + Clone + 'static {
    /// Connect to an existing mailbox by ID
    pub fn connect(channel_id: u64) -> Self {
        Self {
            channel_id,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Send a message (automatically handles cross-privilege)
    pub fn send(&self, _msg: &T) {
        // The actual TypedMailbox implementation will:
        // 1. Detect privilege level difference
        // 2. Use appropriate transport (shared memory, syscall, etc.)
        // 3. Handle serialization if needed

        // Stub: would use actual IPC mechanism
    }

    /// Receive a message (blocks)
    pub fn receive(&self) -> T {
        // Stub: would use actual IPC mechanism
        panic!("TypedMailbox not yet connected to kernel");
    }

    /// Try to receive without blocking
    pub fn try_receive(&self) -> Option<T> {
        // Stub: would use actual IPC mechanism
        None
    }
}

/// Global runtime service mailbox (lazy initialized)
static mut RUNTIME_MAILBOX: Option<TypedMailbox<RuntimeRequest>> = None;
static mut RESPONSE_MAILBOX: Option<TypedMailbox<RuntimeResponse>> = None;

/// Initialize runtime mailboxes (called once at startup)
fn init_runtime_mailbox() {
    unsafe {
        if (*addr_of_mut!(RUNTIME_MAILBOX)).is_none() {
            *addr_of_mut!(RUNTIME_MAILBOX) = Some(TypedMailbox::connect(mailbox_ids::RUNTIME_SERVICE));

            // Create response mailbox for this tasklet
            // In real implementation, would get tasklet ID from kernel
            let tasklet_id = 0; // Placeholder
            *addr_of_mut!(RESPONSE_MAILBOX) = Some(TypedMailbox::connect(
                mailbox_ids::WAKER_BASE + tasklet_id
            ));
        }
    }
}

/// Submit an async task to the kernel runtime via TypedMailbox
#[cfg_attr(target_os = "tantraos", lang = "tantraos_block_on")]
pub fn block_on<F: Future>(future: F) -> F::Output {
    init_runtime_mailbox();

    // Get the runtime mailbox
    let runtime_mailbox = unsafe {
        (*addr_of_mut!(RUNTIME_MAILBOX)).as_ref().unwrap()
    };

    let response_mailbox = unsafe {
        (*addr_of_mut!(RESPONSE_MAILBOX)).as_ref().unwrap()
    };

    // Send BlockOn request to kernel runtime
    let request = RuntimeRequest::BlockOn {
        future_ptr: &future as *const _ as usize,
    };

    runtime_mailbox.send(&request);

    // Wait for completion response
    loop {
        match response_mailbox.receive() {
            RuntimeResponse::Completed => {
                // Future completed successfully
                // In real implementation, would extract the result
                panic!("TantraOS async runtime not fully connected yet");
            }
            RuntimeResponse::Error { code } => {
                panic!("Runtime error: {}", code);
            }
            _ => {
                // Continue waiting
            }
        }
    }
}

/// Yield execution back to the kernel scheduler via TypedMailbox
#[allow(dead_code)]
pub fn yield_now() {
    init_runtime_mailbox();

    if let Some(mailbox) = unsafe { (*addr_of_mut!(RUNTIME_MAILBOX)).as_ref() } {
        mailbox.send(&RuntimeRequest::Yield);
    }
}

/// Exit the current tasklet
#[allow(dead_code)]
pub fn exit_tasklet(status: i32) -> ! {
    init_runtime_mailbox();

    if let Some(mailbox) = unsafe { (*addr_of_mut!(RUNTIME_MAILBOX)).as_ref() } {
        mailbox.send(&RuntimeRequest::Exit { status });
    }

    // Should never return
    loop {
        core::hint::spin_loop();
    }
}

/// Entry point for async main functions
/// This communicates with kernel runtime via TypedMailbox
pub fn tantraos_async_main_entry<F: Future>(future: F) -> F::Output {
    // Register with kernel runtime via TypedMailbox
    block_on(future)
}

/// Run async main function - used by Termination trait implementation
pub fn run_async_main<F: Future>(future: F) -> F::Output {
    // This is the same as block_on but with a clearer name for the context
    block_on(future)
}

/// Special entry point for compiler-generated async main support
/// This is called directly by the compiler for async main functions
#[unsafe(no_mangle)]
#[allow(improper_ctypes_definitions)]
pub unsafe extern "C" fn __tantraos_async_main_wrapper(
    _future_ptr: *mut u8,  // Pointer to the Future returned by main
) -> i32 {
    // Safety: This function is only called by compiler-generated code
    // with a valid Future pointer

    // For now, we can't properly execute the future without proper type info
    // In a full implementation, we'd need to:
    // 1. Type-erase the future into a dyn Future
    // 2. Pass it to block_on
    // 3. Return the result

    // TODO: Implement actual async execution when we have proper type erasure
    // For now, just acknowledge we received the future

    // Return 0 for success
    0
}

/// Dummy type to re-export for the Termination trait implementation
/// This doesn't actually implement anything since we handle it through the compiler bypass
pub struct AsyncMainTermination;

/// Waker implementation that notifies via TypedMailbox
#[allow(dead_code)]
pub struct MailboxWaker {
    task_id: u64,
    mailbox: TypedMailbox<RuntimeResponse>,
}

impl MailboxWaker {
    pub fn new(task_id: u64) -> Self {
        Self {
            task_id,
            mailbox: TypedMailbox::connect(mailbox_ids::WAKER_BASE + task_id),
        }
    }

    pub fn wake(&self) {
        // Send wake notification to runtime
        self.mailbox.send(&RuntimeResponse::Wake {
            task_id: self.task_id,
        });
    }
}

// Force instantiation of Termination trait for unit type
// This ensures MIR is generated for cross-compilation
#[doc(hidden)]
pub fn __force_termination_instantiation() {
    use crate::process::Termination;

    // This function forces the compiler to generate MIR for the Termination trait
    // implementation for the unit type (), which is required for main() functions
    let _exit_code = ().report();
}