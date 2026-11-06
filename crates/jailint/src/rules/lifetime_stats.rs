//! Opt-in phase attribution; normal lint runs do not read a clock or format records.
//!
//! Enable `JAILINT_LIFETIME_STATS=1` and group `lifetime-phase` records by phase and
//! procedure. Collect end-to-end latency separately with this tracing disabled.
use jaic::sema::ProcId;

pub(super) struct Timer(Option<std::time::Instant>);

impl Timer {
    pub(super) fn start() -> Self {
        Self(
            jacket_config::env::LIFETIME_STATS
                .is_set()
                .then(std::time::Instant::now),
        )
    }

    pub(super) fn finish(self, phase: &str, procedure: Option<ProcId>, items: usize) {
        if let Some(start) = self.0 {
            eprintln!(
                "lifetime-phase phase={phase} proc={} items={items} wall_us={:.3}",
                procedure.map_or_else(|| "all".into(), |p| p.0.to_string()),
                start.elapsed().as_secs_f64() * 1_000_000.0,
            );
        }
    }
}
