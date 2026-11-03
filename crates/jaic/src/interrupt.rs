//! Lets an editor ask a check that is under way to stop.
//!
//! A language server checks the open files after an edit, but a request that arrives meanwhile
//! (completion, hover) must not wait for the check to finish. The server [`arm`]s a check it can
//! start again later and [`request`]s a stop when input arrives. The type checker asks
//! [`should_stop`] before each procedure body and, when it says yes, fails that body at once:
//! the check ends through its ordinary error returns, and [`disarm`] tells the caller the result
//! is not to be used. Nothing is armed in the compiler's own command line, where the question
//! is one relaxed load of a flag that stays clear.
use std::sync::atomic::{AtomicU8, AtomicU32, Ordering::Relaxed};

const ARMED: u8 = 1;
const REQUESTED: u8 = 2;
const FIRED: u8 = 4;

static STATE: AtomicU8 = AtomicU8::new(0);

/// Questions left before an armed check is stopped (0: no countdown). For tests, which need a
/// check to stop in the middle.
static COUNTDOWN: AtomicU32 = AtomicU32::new(0);

/// Stops the next armed check at its `polls`-th question, as if input had arrived then.
#[doc(hidden)]
pub fn stop_after(polls: u32) {
    COUNTDOWN.store(polls, Relaxed);
}

/// Allows a stop to take effect.
pub fn arm() {
    STATE.store(ARMED | (STATE.load(Relaxed) & REQUESTED), Relaxed);
}

/// Ends the armed span. True when a check noticed a stop request while it was armed: whatever it
/// produced is incomplete.
pub fn disarm() -> bool {
    STATE.fetch_and(!(ARMED | FIRED), Relaxed) & FIRED != 0
}

/// Asks (`true`) a check that is armed to stop, or clears the request (`false`).
pub fn request(stop: bool) {
    if stop {
        STATE.fetch_or(REQUESTED, Relaxed);
    } else {
        STATE.fetch_and(!REQUESTED, Relaxed);
    }
}

/// Whether the armed check has been asked to stop; once true it stays true until [`disarm`], so
/// the first caller to notice and every one after it agree.
#[inline]
pub fn should_stop() -> bool {
    let state = STATE.load(Relaxed);
    if state & (ARMED | REQUESTED) != (ARMED | REQUESTED) && state & FIRED == 0 {
        // Only an armed check counts down, so the unarmed path stays one load.
        if state & ARMED == 0
            || COUNTDOWN.load(Relaxed) == 0
            || COUNTDOWN.fetch_sub(1, Relaxed) != 1
        {
            return false;
        }
    }
    STATE.fetch_or(FIRED, Relaxed);
    true
}

/// Whether a stoppable check is under way.
pub fn armed() -> bool {
    STATE.load(Relaxed) & ARMED != 0
}

/// Whether a stop has been noticed in the armed span: callers stop starting work.
pub fn fired() -> bool {
    STATE.load(Relaxed) & FIRED != 0
}
