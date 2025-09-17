//! TantraOS panic handling implementation
//!
//! This module provides panic handling that communicates with the kernel
//! via TypedMailbox for proper tasklet termination.

#![allow(dead_code)]

use crate::panic::PanicHookInfo;
use super::runtime::exit_tasklet;

/// Handle panic by notifying the kernel and terminating the tasklet
pub fn panic_handler(_info: &PanicHookInfo<'_>) -> ! {
    // Try to send panic information to kernel
    // The kernel can then log it or take appropriate action

    // For now, just exit with error code
    // TODO: Send detailed panic info via mailbox
    exit_tasklet(-1)
}

// Note: The panic handler lang item is defined in lib.rs, not here
// to avoid duplicate lang item errors

/// Set up the panic hook for TantraOS
pub fn set_panic_hook() {
    // This will be called during std initialization
    // to set up TantraOS-specific panic handling
}