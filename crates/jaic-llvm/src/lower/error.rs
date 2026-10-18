//! The error type lowering reports.
use inkwell::builder::BuilderError;

/// Backend error; converted to a `String` at the crate boundary.
pub struct Error(pub(super) String);

impl From<BuilderError> for Error {
    fn from(e: BuilderError) -> Self {
        Error(format!("LLVM builder error: {e}"))
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error(e.to_string())
    }
}
