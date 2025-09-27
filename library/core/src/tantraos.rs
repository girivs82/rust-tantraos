//! TantraOS-specific core library extensions
//!
//! This module provides TantraOS-specific attributes and runtime support
//! for async tasklets, including EL0 (userspace) execution.

/// Marks an async function as an EL0 tasklet
///
/// Functions marked with this attribute will be compiled to run at
/// EL0 (userspace) with limited privileges. They communicate with
/// the kernel via mailbox IPC and use SVC for yielding.
///
/// # Example
///
/// ```no_run
/// #[tantraos::el0_tasklet]
/// async fn user_driver() {
///     loop {
///         // Driver logic here
///         yield_now().await;
///     }
/// }
/// ```
#[cfg(target_os = "tantraos")]
// pub use tantraos_macros::el0_tasklet; // TODO: Add tantraos_macros crate

/// Block on a future (TantraOS runtime)
///
/// This is the TantraOS-specific block_on implementation that
/// integrates with the kernel's async runtime.
#[cfg(target_os = "tantraos")]
#[lang = "tantraos_block_on"]
#[stable(feature = "tantraos_runtime", since = "1.0.0")]
pub fn block_on<F: Future>(_future: F) -> F::Output {
    // This is a stub - the actual implementation is in the kernel
    // The compiler will replace calls to this with the actual runtime
    unsafe { core::hint::unreachable_unchecked() }
}

/// Yield control back to the scheduler
///
/// This allows cooperative multitasking by voluntarily yielding
/// the CPU to other tasklets.
#[cfg(target_os = "tantraos")]
#[stable(feature = "tantraos_runtime", since = "1.0.0")]
pub async fn yield_now() {
    struct YieldFuture {
        yielded: bool,
    }

    impl Future for YieldFuture {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.yielded {
                Poll::Ready(())
            } else {
                self.yielded = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    YieldFuture { yielded: false }.await
}

/// EL0 runtime support
#[cfg(all(target_os = "tantraos", target_arch = "aarch64"))]
#[stable(feature = "tantraos_runtime", since = "1.0.0")]
pub mod el0_runtime {
    use core::arch::asm;
    use core::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};
    use core::future::Future;
    use core::pin::Pin;

    /// Yield to EL1 via SVC
    #[inline(always)]
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub fn svc_yield() {
        unsafe {
            asm!(
                "svc #2",
                options(nostack, preserves_flags)
            );
        }
    }

    /// Return to EL1 with a value
    #[inline(always)]
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub fn svc_return(value: u64) -> ! {
        unsafe {
            asm!(
                "svc #1",
                in("x0") value,
                options(noreturn, nostack)
            );
        }
    }

    /// Wake a tasklet via SVC
    #[inline(always)]
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub fn svc_wake(tasklet_id: u64) {
        unsafe {
            asm!(
                "svc #3",
                in("x0") tasklet_id,
                options(nostack, preserves_flags)
            );
        }
    }

    /// EL0 waker vtable
    static EL0_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
        el0_clone,
        el0_wake,
        el0_wake_by_ref,
        el0_drop,
    );

    unsafe fn el0_clone(data: *const ()) -> RawWaker {
        RawWaker::new(data, &EL0_WAKER_VTABLE)
    }

    unsafe fn el0_wake(data: *const ()) {
        let tasklet_id = data as u64;
        svc_wake(tasklet_id);
    }

    unsafe fn el0_wake_by_ref(data: *const ()) {
        unsafe { el0_wake(data); }
    }

    unsafe fn el0_drop(_: *const ()) {
        // No-op
    }

    /// Create an EL0 waker
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub fn create_el0_waker(tasklet_id: u64) -> Waker {
        unsafe {
            #[allow(fuzzy_provenance_casts)]
            Waker::from_raw(RawWaker::new(
                tasklet_id as *const (),
                &EL0_WAKER_VTABLE,
            ))
        }
    }

    /// EL0 Future polling wrapper
    ///
    /// This function polls a future at EL0, handling the cross-privilege
    /// communication via SVC instructions.
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub unsafe fn poll_el0_future<F: Future>(
        future: *mut F,
        tasklet_id: u64,
    ) -> Poll<F::Output> {
        // Create EL0-compatible waker
        let waker = create_el0_waker(tasklet_id);
        let mut context = Context::from_waker(&waker);

        // Pin the future
        let future_ref = unsafe { &mut *future };
        let pinned = unsafe { Pin::new_unchecked(future_ref) };

        // Poll the future
        pinned.poll(&mut context)
    }

    /// EL0 tasklet entry point
    ///
    /// This is called by the kernel when starting an EL0 tasklet.
    /// It sets up the async runtime and begins polling the future.
    #[unsafe(no_mangle)]
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub unsafe extern "C" fn __tantraos_el0_entry(
        future_ptr: *mut u8,
        poll_fn: unsafe extern "C" fn(*mut u8, u64) -> u8,
        tasklet_id: u64,
    ) -> ! {
        // Simple polling loop
        loop {
            // Poll the future using the provided function pointer
            let result = unsafe { poll_fn(future_ptr, tasklet_id) };

            if result == 0 {
                // Pending - yield to scheduler
                svc_yield();
            } else {
                // Ready - return to EL1
                svc_return(result as u64);
            }
        }
    }

    /// Generic poll wrapper for the compiler to use
    #[stable(feature = "tantraos_runtime", since = "1.0.0")]
    pub unsafe extern "C" fn __tantraos_poll_wrapper<F: Future>(
        future_ptr: *mut u8,
        tasklet_id: u64,
    ) -> u8 {
        let future = future_ptr as *mut F;

        match unsafe { poll_el0_future(future, tasklet_id) } {
            Poll::Ready(_) => 1,
            Poll::Pending => 0,
        }
    }
}

use core::future::Future;
use core::task::{Context, Poll};
use core::pin::Pin;