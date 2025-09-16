//! Target configuration for TantraOS on AArch64
//!
//! TantraOS is an async-first operating system where everything is a tasklet.
//! This target uses std-tantraos as the standard library and provides
//! zero-boilerplate async tasklet development.

use crate::spec::{base, Cc, LinkerFlavor, Lld, Target, TargetOptions};

pub(crate) fn target() -> Target {
    Target {
        llvm_target: "aarch64-unknown-none".into(),
        metadata: crate::spec::TargetMetadata {
            description: Some("TantraOS on AArch64 (async-first OS)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true), // We DO have std (std-tantraos)
        },
        pointer_width: 64,
        data_layout: "e-m:e-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32".into(),
        arch: "aarch64".into(),

        options: TargetOptions {
            features: "+v8a,+strict-align,+neon,+fp-armv8".into(),
            max_atomic_width: Some(128),
            supported_sanitizers: base::SanitizerSet::empty(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),

            // TantraOS-specific options
            os: "tantraos".into(),
            env: "tasklet".into(),  // Indicates tasklet environment
            vendor: "tantraos".into(),

            // Custom entry point for TNF tasklets
            entry_name: "tasklet_main".into(),

            // Async-first configuration
            panic_strategy: base::PanicStrategy::Abort,
            default_adjusted_cabi: Some(base::cabi::Abi::C { unwind: false }),

            // No runtime dependencies on libc
            no_default_libraries: true,
            position_independent_executables: false,
            static_position_independent_executables: false,
            needs_plt: false,
            relro_level: base::RelroLevel::Off,
            stack_probes: base::StackProbeType::Inline,
            eliminate_frame_pointer: false,

            // TantraOS uses its own allocator
            has_thread_local: false,
            tls_model: base::TlsModel::Emulated,

            // Disable Rust's std, we'll provide std-tantraos
            no_std: false,  // We HAVE std, just our own

            // TNF binary format settings
            exe_suffix: ".tnf".into(),
            staticlib_suffix: ".a".into(),
            dll_suffix: ".so".into(),

            // Custom linking for TNF format
            pre_link_args: base::TargetOptions::link_args(
                LinkerFlavor::Gnu(Cc::No, Lld::Yes),
                &["--entry=tasklet_main"],
            ),

            ..Default::default()
        },
    }
}