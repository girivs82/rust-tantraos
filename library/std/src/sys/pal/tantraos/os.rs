//! TantraOS-specific functionality

#![allow(dead_code)]

use crate::ffi::{OsStr, OsString};
use crate::io;
use crate::path::{Path, PathBuf};

pub fn errno() -> i32 {
    // TODO: Get actual errno from TantraOS kernel
    0
}

pub fn error_string(errno: i32) -> String {
    // TODO: Convert TantraOS error codes to strings
    format!("TantraOS error {}", errno)
}

pub fn getcwd() -> io::Result<PathBuf> {
    // TODO: Implement actual getcwd via TantraOS filesystem
    Ok(PathBuf::from("/"))
}

pub fn chdir(p: &Path) -> io::Result<()> {
    let _ = p;
    // TODO: Implement chdir via TantraOS filesystem
    Err(super::unsupported_err())
}

pub fn getenv(k: &OsStr) -> Option<OsString> {
    let _ = k;
    // TODO: Implement environment variables for TantraOS
    None
}

pub fn setenv(k: &OsStr, v: &OsStr) -> io::Result<()> {
    let _ = (k, v);
    // TODO: Implement environment variables for TantraOS
    Err(super::unsupported_err())
}

pub fn unsetenv(n: &OsStr) -> io::Result<()> {
    let _ = n;
    // TODO: Implement environment variables for TantraOS
    Err(super::unsupported_err())
}

pub fn current_exe() -> io::Result<PathBuf> {
    // TODO: Get current executable path from TantraOS
    Ok(PathBuf::from("/tasklet.tnf"))
}

pub fn home_dir() -> Option<PathBuf> {
    // TODO: Get home directory from TantraOS
    Some(PathBuf::from("/home"))
}

pub fn temp_dir() -> PathBuf {
    // TODO: Get temp directory from TantraOS
    PathBuf::from("/tmp")
}

pub struct Env {
    // Empty for now - TantraOS will have its own environment model
}

impl Env {
    pub fn new() -> Env {
        Env {}
    }
}

impl Iterator for Env {
    type Item = (OsString, OsString);

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub fn env() -> Env {
    Env::new()
}

pub fn getpid() -> u32 {
    // TODO: Get process/tasklet ID from TantraOS kernel
    1
}

pub fn exit(code: i32) -> ! {
    // TODO: Exit via TantraOS kernel syscall
    let _ = code;
    core::intrinsics::abort();
}

// Path-related functions
pub struct SplitPaths<'a> {
    inner: &'a OsStr,
}

impl<'a> Iterator for SplitPaths<'a> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<PathBuf> {
        None // TODO: Implement path splitting for TantraOS
    }
}

pub fn split_paths(unparsed: &OsStr) -> SplitPaths<'_> {
    SplitPaths { inner: unparsed }
}

pub struct JoinPathsError {
    inner: (),
}

impl core::fmt::Display for JoinPathsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "JoinPathsError".fmt(f)
    }
}

impl JoinPathsError {
    pub fn description(&self) -> &str {
        "path separator found in input"
    }
}

impl core::fmt::Debug for JoinPathsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "JoinPathsError".fmt(f)
    }
}

pub fn join_paths<I, T>(paths: I) -> Result<OsString, JoinPathsError>
where
    I: Iterator<Item = T>,
    T: AsRef<OsStr>,
{
    let _ = paths;
    // TODO: Implement path joining for TantraOS
    Err(JoinPathsError { inner: () })
}


