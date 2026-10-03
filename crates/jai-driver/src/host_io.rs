//! Explicit host policy, atomic compiler coordination and source binding bridge.
mod provider;
pub use provider::*;
mod original_input_receipts;
mod original_inputs;
pub use original_inputs::OriginalInputPolicy;
mod compiler_session;
pub use compiler_session::FileCompilerSession;
mod source;
