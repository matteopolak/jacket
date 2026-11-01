//! The counters `-dev_body_probe` prints, and the test hooks that read them.
#[cfg(all(test, feature = "speculative-bodies"))]
use super::{ON, Stats};
use super::{Reason, STATS, body_probe, enabled};
use std::time::Duration;

/// Print the counters.
pub fn report() {
    if !enabled() || !body_probe::enabled() {
        return;
    }
    STATS.with(|s| {
        let s = s.borrow();
        let aborted: u64 = s.aborted.values().map(|a| a.0).sum();
        eprintln!(
            "== speculative bodies: {} batches, {} bodies batched, {} committed, {} abandoned",
            s.batches, s.batched, s.committed, aborted
        );
        let pct = |n: u64| 100.0 * n as f64 / s.batched.max(1) as f64;
        eprintln!(
            "  committed {:.1}% of batched; moved {} scopes, {} entities",
            pct(s.committed),
            s.scopes_moved,
            s.entities_moved
        );
        eprintln!(
            "  {} first-use steps run at a body's turn; {} bodies committed after that, {} ({:.1}%) on the first try",
            s.resolved_ahead,
            s.recovered,
            s.committed - s.recovered,
            pct(s.committed - s.recovered)
        );
        eprintln!("-- abandoned by first blocker:");
        let mut rows: Vec<_> = s.aborted.iter().collect();
        rows.sort_by_key(|(_, (n, _))| std::cmp::Reverse(*n));
        for (reason, (n, time)) in rows {
            let name = match reason {
                Reason::Wrote(i) => format!("wrote {}", body_probe::write_name(*i)),
                Reason::Resolving(i) => format!(
                    "wrote {} while resolving an entity",
                    body_probe::write_name(*i)
                ),
                Reason::Failed => "reported an error".into(),
                Reason::Immovable(why) => format!("immovable: {why}"),
                Reason::Invalidated => "invalidated by a serial body".into(),
                Reason::LoweredMeanwhile => "lowered on demand by an earlier body".into(),
                Reason::Unsupported => "not eligible".into(),
            };
            eprintln!(
                "  {name:<44} {n:>8} ({:>4.1}%)  wasted {time:>10.3?}",
                pct(*n)
            );
        }
        let per = |t: Duration, n: u64| t.as_secs_f64() * 1e6 / n.max(1) as f64;
        eprintln!("-- time:");
        eprintln!(
            "  view path, committed   {:>10.3?} ({:.1} us/body)",
            s.view_time,
            per(s.view_time, s.committed)
        );
        eprintln!(
            "  view path, abandoned   {:>10.3?} ({:.1} us/body)",
            s.view_wasted,
            per(s.view_wasted, aborted)
        );
        eprintln!(
            "  commit                 {:>10.3?} ({:.2} us/body)",
            s.commit_time,
            per(s.commit_time, s.committed)
        );
        eprintln!(
            "  serial path (batched)  {:>10.3?} ({:.1} us/body)",
            s.serial_time,
            per(s.serial_time, s.serial_bodies)
        );
    });
}

/// How a thread's bodies went so far.
#[cfg(all(test, feature = "speculative-bodies"))]
pub(in crate::sema) struct Counts {
    pub batched: u64,
    pub committed: u64,
    pub resolved_ahead: u64,
    pub recovered: u64,
}

/// Switch speculation on or off for this thread and zero the counters (tests).
#[cfg(all(test, feature = "speculative-bodies"))]
pub(in crate::sema) fn force(on: bool) {
    ON.with(|o| o.set(on));
    STATS.with(|s| *s.borrow_mut() = Stats::default());
}

#[cfg(all(test, feature = "speculative-bodies"))]
pub(in crate::sema) fn counts() -> Counts {
    STATS.with(|s| {
        let s = s.borrow();
        Counts {
            batched: s.batched,
            committed: s.committed,
            resolved_ahead: s.resolved_ahead,
            recovered: s.recovered,
        }
    })
}
