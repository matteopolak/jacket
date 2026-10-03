//! Optional native embedding adapter. Host services are explicit trusted grants.
use crate::{HostServices, Platform};
pub use jai_native_source::{Filesystem, NativeSourceSnapshot};

pub struct NativePlatform<H: HostServices> {
    source: NativeSourceSnapshot,
    host: H,
}
impl<H: HostServices> NativePlatform<H> {
    pub fn new(source: NativeSourceSnapshot, host: H) -> Self {
        Self {
            source,
            host,
        }
    }
    pub fn host(&self) -> &H {
        &self.host
    }
    pub fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }
}
impl<H: HostServices> Platform for NativePlatform<H> {
    fn source_files(&self) -> &dyn jai_source::SourceProvider {
        &self.source
    }
    fn host_services(&mut self) -> &mut dyn HostServices {
        &mut self.host
    }
}
