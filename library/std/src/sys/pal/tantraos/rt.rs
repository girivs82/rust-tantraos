//! Runtime support for TantraOS
//!
//! This module provides the TantraOS-specific implementation of the runtime
//! entry point that handles async main functions transparently.

use crate::panic;
use crate::process::{ExitCode, Termination};

// Import our runtime module
use super::runtime;

/// TantraOS-specific lang_start implementation
/// This is the actual entry point for TantraOS programs
#[stable(feature = "tantraos_lang_start", since = "1.92.0")]
pub fn lang_start<T: Termination + 'static>(
    main: fn() -> T,
    argc: isize,
    argv: *const *const u8,
    sigpipe: u8,
) -> isize {
    // Initialize TantraOS runtime
    unsafe {
        super::init(argc, argv, sigpipe);
    }

    // Run main and handle panics
    let result = panic::catch_unwind(|| {
        let ret_val = main();
        ret_val.report()
    }).unwrap_or(ExitCode::from(101));

    // Cleanup
    crate::rt::cleanup();

    result.to_i32() as isize
}

/// Entry point attribute for TantraOS
/// This allows users to write just `async fn main()` without any attributes
#[doc(hidden)]
#[macro_export]
#[stable(feature = "tantraos_main", since = "1.92.0")]
macro_rules! tantraos_main {
    () => {
        // This macro is automatically injected for TantraOS targets
        // It provides the necessary boilerplate for async main
        #[no_mangle]
        pub extern "C" fn tasklet_main() -> ! {
            // Get the async main function
            let main_future = async {
                main().await
            };

            // Run it using the TantraOS runtime
            $crate::sys::pal::tantraos::runtime::block_on(main_future);

            // Exit the tasklet
            $crate::process::exit(0);
        }
    };
}

/// Support for async main in TantraOS
/// This is used when the compiler detects an async main function
#[cfg(target_os = "tantraos")]
#[doc(hidden)]
#[allow(dead_code)]
pub fn run_async_main<F: core::future::Future>(future: F) -> F::Output {
    runtime::block_on(future)
}