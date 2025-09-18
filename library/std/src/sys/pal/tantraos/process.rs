//! Process management for TantraOS
//!
//! TantraOS uses tasklets instead of traditional processes.

#![allow(dead_code)]

use crate::io;
use crate::sys::pipe::AnonPipe;
use crate::sys::runtime;

pub struct Command {
    // TantraOS doesn't have traditional processes
}

impl Command {
    pub fn new(_program: &crate::ffi::OsStr) -> Command {
        Command {}
    }

    pub fn arg(&mut self, _arg: &crate::ffi::OsStr) -> &mut Command {
        self
    }

    pub fn args(&mut self, _args: &[&crate::ffi::OsStr]) -> &mut Command {
        self
    }

    pub fn env(&mut self, _key: &crate::ffi::OsStr, _val: &crate::ffi::OsStr) -> &mut Command {
        self
    }

    pub fn env_clear(&mut self) -> &mut Command {
        self
    }

    pub fn cwd(&mut self, _dir: &crate::ffi::OsStr) -> &mut Command {
        self
    }

    pub fn stdin(&mut self, _stdin: Stdio) -> &mut Command {
        self
    }

    pub fn stdout(&mut self, _stdout: Stdio) -> &mut Command {
        self
    }

    pub fn stderr(&mut self, _stderr: Stdio) -> &mut Command {
        self
    }

    pub fn spawn(&mut self, _default: Stdio, _needs_stdin: bool) -> io::Result<Process> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "TantraOS doesn't support traditional process spawning",
        ))
    }
}

pub struct Stdio;

impl Stdio {
    pub const fn null() -> Stdio {
        Stdio
    }

    pub const fn inherit() -> Stdio {
        Stdio
    }

    pub fn piped() -> Stdio {
        Stdio
    }

    pub fn to_child_stdio(&self, _readable: bool) -> io::Result<ChildStdio> {
        Ok(ChildStdio::Inherit)
    }
}

pub enum ChildStdio {
    Inherit,
    Null,
    Owned(AnonPipe),
}

pub struct Process {
    handle: u64,
}

impl Process {
    pub fn id(&self) -> u32 {
        self.handle as u32
    }

    pub fn kill(&mut self) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "TantraOS tasklets cannot be killed",
        ))
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        Ok(ExitStatus { code: 0 })
    }

    pub fn try_wait(&mut self) -> io::Result<Option<ExitStatus>> {
        Ok(Some(ExitStatus { code: 0 }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitStatus {
    code: i32,
}

impl ExitStatus {
    pub fn exit_code(&self) -> Option<i32> {
        Some(self.code)
    }

    pub fn code(&self) -> Option<i32> {
        Some(self.code)
    }

    pub fn success(&self) -> bool {
        self.code == 0
    }
}

impl Default for ExitStatus {
    fn default() -> Self {
        ExitStatus { code: 0 }
    }
}

impl From<i32> for ExitStatus {
    fn from(code: i32) -> Self {
        ExitStatus { code }
    }
}

#[stable(feature = "tantraos_process", since = "1.0.0")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExitCode(i32);

impl ExitCode {
    #[stable(feature = "tantraos_process", since = "1.0.0")]
    pub const SUCCESS: ExitCode = ExitCode(0);
    #[stable(feature = "tantraos_process", since = "1.0.0")]
    pub const FAILURE: ExitCode = ExitCode(1);

    #[stable(feature = "tantraos_process", since = "1.0.0")]
    pub fn as_i32(self) -> i32 {
        self.0
    }

    #[stable(feature = "tantraos_process", since = "1.0.0")]
    pub fn from_raw(code: i32) -> Self {
        ExitCode(code)
    }
}

#[stable(feature = "tantraos_process", since = "1.0.0")]
impl From<u8> for ExitCode {
    fn from(code: u8) -> Self {
        ExitCode(code as i32)
    }
}

#[stable(feature = "tantraos_process", since = "1.0.0")]
impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> i32 {
        code.0
    }
}

pub fn exit(code: i32) -> ! {
    runtime::exit_tasklet(code)
}

pub fn abort() -> ! {
    // For TantraOS, abort is the same as exit with failure
    runtime::exit_tasklet(1)
}

pub fn getpid() -> u32 {
    // TantraOS doesn't have PIDs, return a placeholder
    1
}

pub fn id() -> u32 {
    getpid()
}