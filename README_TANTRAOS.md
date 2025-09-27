# Rust Compiler for TantraOS

This is a custom fork of the Rust compiler that adds support for TantraOS, an async-first operating system.

## Features

- **TantraOS Target**: Adds `aarch64-tantraos` as a supported compilation target
- **Async Runtime Integration**: Built-in support for TantraOS async runtime
- **Standard Library**: Custom std implementation for TantraOS system calls
- **TNF Binary Format**: Generates TantraOS Native Format executables

## Building

```bash
# Clone the repository
git clone https://github.com/girivs82/rust-tantraos.git
cd rust-tantraos

# Build the compiler (stage 1)
env BOOTSTRAP_SKIP_TARGET_SANITY=1 ./x.py build --stage 1 --target aarch64-tantraos

# The compiler will be in:
# build/aarch64-apple-darwin/stage1/bin/rustc
```

## Usage

```bash
# Compile a TantraOS tasklet to TNF binary
rustc --edition 2024 --target aarch64-tantraos \
      --sysroot $RUST_SYSROOT \
      your_app.rs -o your_app.tnf
```

## Zero-Boilerplate Async Tasklets

With the proc-macro approach (see TantraOS main repo), you can write:

```rust
#[tasklet_main]
async fn main() {
    println!("Hello from TantraOS!");
}
```

No `#![no_std]`, `#![no_main]`, or panic handler boilerplate required!

## Maintenance

This fork needs to be periodically synced with upstream Rust:

```bash
# Add upstream remote if not already added
git remote add upstream https://github.com/rust-lang/rust.git

# Fetch and merge upstream changes
git fetch upstream
git checkout tantraos-target
git merge upstream/master

# Resolve any conflicts in TantraOS-specific files:
# - compiler/rustc_target/src/spec/targets/aarch64_tantraos.rs
# - library/std/src/sys/pal/tantraos/*
# - library/std/src/os/tantraos/*
```

**Last sync:** 2025-09-18 - Merged cleanly with no conflicts

## TantraOS-Specific Changes

Key files modified/added for TantraOS support:

- `compiler/rustc_target/src/spec/targets/aarch64_tantraos.rs` - Target specification
- `library/std/src/sys/pal/tantraos/` - System abstraction layer
- `library/std/src/os/tantraos/` - OS-specific APIs
- `library/std/src/sys/pal/tantraos/runtime.rs` - Async runtime integration

## Known Issues

- **Termination Trait MIR**: Regular `fn main()` doesn't work due to missing MIR generation for cross-compilation
  - Workaround: Use `#![no_std]` + `#![no_main]` with `_start` entry point
  - Solution: Use the `#[tasklet_main]` proc-macro

## Contributing

Please submit issues and PRs to the main TantraOS repository for OS-specific changes.
For Rust compiler issues, submit to upstream Rust first if not TantraOS-specific.

## License

Licensed under the same terms as Rust itself (MIT/Apache 2.0).