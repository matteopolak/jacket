use jai_vm::{CompilerOutputStream, SourceOrigin, host_effects::*};
use std::fmt;

pub use jai_vm::host_effects::HostCapability;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlatformError {
    UnsupportedHostCapability(HostCapability),
    Host(HostError),
}
impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedHostCapability(capability) => {
                write!(f, "unsupported host capability: {capability:?}")
            }
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for PlatformError {
}
impl From<HostError> for PlatformError {
    fn from(error: HostError) -> Self {
        Self::Host(error)
    }
}

pub trait HostServices: HostEffects {
    fn capabilities(&self) -> &[HostCapability];
    fn file_scope(&self) -> Option<FilePathScope>;
    fn poll_host_request(&mut self, origin: &SourceOrigin, key: HostRequestKey) -> HostOutcome;
    /// Service only an already validated genuine ticket owned by a suspended journal.
    fn service_host_request(
        &mut self,
        _: &SourceOrigin,
        _: HostRequestKey,
    ) -> Result<bool, HostError> {
        Err(HostError::UnknownCapability)
    }
    fn suspend(&mut self, origin: &SourceOrigin) -> Result<(), HostError>;
    fn resume(&mut self, origin: &SourceOrigin) -> Result<(), HostError>;
    fn cancel(&mut self, origin: &SourceOrigin) -> Result<(), HostError>;
    fn write_console(
        &mut self,
        stream: CompilerOutputStream,
        bytes: Vec<u8>,
    ) -> Result<(), PlatformError>;
    fn monotonic_nanoseconds(&mut self) -> Result<u64, PlatformError> {
        Err(PlatformError::UnsupportedHostCapability(
            HostCapability::Clock,
        ))
    }
    /// Validate support before lowering a service request into the VM protocol.
    fn require(&self, capability: HostCapability) -> Result<(), PlatformError> {
        if self.capabilities().contains(&capability) {
            Ok(())
        } else {
            Err(PlatformError::UnsupportedHostCapability(capability))
        }
    }
}

/// Source snapshots and effect grants belong to one embedding-owned platform.
pub trait Platform {
    fn source_files(&self) -> &dyn jai_source::SourceProvider;
    fn host_services(&mut self) -> &mut dyn HostServices;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConsoleOutput {
    pub stream: CompilerOutputStream,
    pub bytes: Vec<u8>,
}
