//! The parts of `jacket` that other programs reuse: dependency resolution (`dep`) and the git,
//! HTTP and prerequisite checks it stands on. The `jacket` command is the binary of this package.
#![forbid(unsafe_code)]

pub mod dep;
pub mod git;
pub mod net;
pub mod prerequisite;
