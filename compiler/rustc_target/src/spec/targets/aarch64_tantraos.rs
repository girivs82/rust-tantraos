//! Target configuration for TantraOS on AArch64
//!
//! TantraOS is an async-first operating system where everything is a tasklet.
//! This target uses std-tantraos as the standard library and provides
//! zero-boilerplate async tasklet development.

use crate::spec::{
    cvs, Cc, LinkerFlavor, Lld, PanicStrategy, RelocModel, RelroLevel, SanitizerSet,
    StackProbeType, Target, TargetMetadata, TargetOptions, TlsModel, FramePointer
};

pub(crate) fn target() -> Target {
    Target {
        llvm_target: "aarch64-unknown-none".into(),
        metadata: TargetMetadata {
            description: Some("TantraOS on AArch64 (async-first OS)".into()),
            tier: Some(3),
            host_tools: Some(false),
            std: Some(true), // We have std support via TantraOS PAL
        },
        pointer_width: 64,
        data_layout: "e-m:e-p270:32:32-p271:32:32-p272:64:64-i8:8:32-i16:16:32-i64:64-i128:128-n32:64-S128-Fn32".into(),
        arch: "aarch64".into(),

        options: TargetOptions {
            features: "+v8a,+strict-align,+neon,+fp-armv8".into(),
            max_atomic_width: Some(128),
            supported_sanitizers: SanitizerSet::empty(),
            linker_flavor: LinkerFlavor::Gnu(Cc::No, Lld::Yes),
            linker: Some("rust-lld".into()),

            // Force static linking - no dynamic libraries
            relocation_model: RelocModel::Static,
            disable_redzone: true,

            // TantraOS-specific options
            os: "tantraos".into(),
            env: "eabi".into(),  // Use standard bare-metal environment
            // vendor defaults to "unknown" - don't override
            families: cvs![],  // TantraOS is NOT Unix - it's its own OS family

            // Custom entry point for TNF tasklets
            entry_name: "tasklet_main".into(),

            // Async-first configuration
            panic_strategy: PanicStrategy::Abort,

            // No runtime dependencies on libc or dynamic linking
            no_default_libraries: true,
            position_independent_executables: false,
            static_position_independent_executables: false,
            relro_level: RelroLevel::Off,
            stack_probes: StackProbeType::Inline,
            frame_pointer: FramePointer::Always,

            // TantraOS uses its own allocator
            has_thread_local: false,
            tls_model: TlsModel::Emulated,

            // TNF binary format settings
            exe_suffix: ".tnf".into(),
            staticlib_suffix: ".a".into(),
            dll_suffix: ".so".into(),

            // Custom linking for TNF format - minimal bare metal linking
            pre_link_args: TargetOptions::link_args(
                LinkerFlavor::Gnu(Cc::No, Lld::No),
                &["--entry=tasklet_main"],
            ),

            ..Default::default()
        },
    }
}