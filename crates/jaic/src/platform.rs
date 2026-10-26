//! What differs between the native compiler and the browser (wasm32) build, named once.
//!
//! Code that behaves differently on wasm asks [`HAS_CLOCK`] or uses [`Stopwatch`] rather than
//! writing its own `cfg(target_arch = "wasm32")`. Larger native-only subsystems (processes,
//! threads, the sandbox) remain separate modules selected where they are declared.
use std::time::Duration;

/// Whether the host has a monotonic clock. The browser build does not.
pub const HAS_CLOCK: bool = cfg!(not(target_arch = "wasm32"));

/// A point in time that reads as zero where there is no clock.
#[derive(Clone, Copy)]
pub struct Stopwatch(#[cfg(not(target_arch = "wasm32"))] std::time::Instant);

impl Stopwatch {
    pub fn start() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        return Stopwatch(std::time::Instant::now());
        #[cfg(target_arch = "wasm32")]
        return Stopwatch();
    }

    pub fn elapsed(self) -> Duration {
        #[cfg(not(target_arch = "wasm32"))]
        return self.0.elapsed();
        #[cfg(target_arch = "wasm32")]
        return Duration::ZERO;
    }
}
