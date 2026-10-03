//! Explicit platform data services shared by native and browser compiler hosts.
mod interface;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub mod native;
mod overlay;
mod vfs;
mod virtual_host;
pub use interface::*;
pub use jai_source::{SourceProvider, normalize_virtual_path};
pub use overlay::SourceOverlay;
pub use vfs::{SharedVfs, VfsLimits, VfsSnapshot};
pub use virtual_host::{VfsHost, VfsHostLimits, VfsPlatform};
