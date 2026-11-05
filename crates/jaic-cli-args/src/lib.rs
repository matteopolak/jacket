//! Compiler option model and inspection, shared by the actual parser and tooling.
#![forbid(unsafe_code)]
pub mod arguments;
pub mod cli;
pub mod options;
pub mod tuning;

use cli::Cli;
use options::{Action, CliError, CliOption, Section};
