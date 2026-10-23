//! Typing-session stability checker: simulated editor traffic against a [`Session`], checked
//! against oracles that do not need to know the right answer.
//!
//! A session opens real source files, records every query over them (definition and hover at each
//! identifier, semantic tokens, diagnostics) and then replays what an editor sends while someone
//! types: one-character incremental changes with queries in between, backspacing, deleting and
//! retyping a token or line, pasting blocks, opening brackets, strings and comments that are closed
//! later, auto-closed bracket pairs, undo, whole-document reverts, and the same edits in a second
//! document the first one loads. Edits stay inside one *region* (a procedure body, or the end of
//! the file), so what lies outside it has a known answer.
//!
//! Oracles:
//! * a: no request panics or exceeds the time budget (a watchdog ends a hung process);
//! * b: with the region's brackets, strings and comments balanced, every probe outside it keeps the
//!   baseline definition, hover, tokens and diagnostics, mapped through the edits;
//! * c: back at the original text every result equals the baseline;
//! * d: every `diff_every`th version equals a fresh [`Session`] opened on the same text, except
//!   while a literal that stopped closing is left out of the analysis on purpose;
//! * e: diagnostics have ordered ranges inside the document on character boundaries, no duplicates;
//! * f: the server's text equals the checker's own copy after every change;
//! * g: text that lexes cleanly has no literal left out of the analysis;
//! * h: asking the baseline's requests twice gives the same answers.
//!
//! Included with `#[path]` by the integration test, the `typing_fuzz` example and the libFuzzer
//! harness, so it uses only the public API.
use jai_language_server::{
    Diagnostic, DiagnosticCode, DocumentUri, Environment, Error, Hover, Limits, Location, Position,
    Range, SemanticToken, Session, TextChange,
};
use jaic::source::FileId;
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

#[path = "typing_session/clock.rs"]
mod clock;
#[path = "typing_session/rig.rs"]
mod rig;
#[path = "typing_session/snapshot.rs"]
mod snapshot;
#[path = "typing_session/text.rs"]
mod text;

use clock::Clock;
use rig::{Rig, here_strings};
pub use snapshot::Query;
use snapshot::{
    Probe, Snapshot, answer, any_omits_literal, omits_literal, probe_at, sanity,
    snapshot_difference, sorted_debug, stale_damage, usage_lint,
};
use text::{balanced, byte_of, identifier_starts, pos_of};

/// Where a session's decisions come from: a seeded generator, or a fuzzer's bytes.
pub trait Choices {
    /// A number below `n` (`n` is at least 1).
    fn below(&mut self, n: usize) -> usize;

    fn chance(&mut self, numerator: usize, denominator: usize) -> bool {
        self.below(denominator) < numerator
    }

    /// Whether the source of choices has run dry (the fuzzer's input is used up).
    fn exhausted(&self) -> bool {
        false
    }
}

/// splitmix64, so a seed reproduces a session on every platform.
pub struct Seeded(pub u64);

impl Choices for Seeded {
    fn below(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        ((z ^ (z >> 31)) % n as u64) as usize
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    /// Scenarios per session (a scenario is one typed statement, retyped line, paste, ...).
    pub scenarios: usize,
    /// Every this many versions, compare with a fresh session (oracle d). 0 turns it off.
    pub diff_every: usize,
    /// Identifier positions compared per version outside the region (oracle b).
    pub probes: usize,
    /// Identifier positions in a full snapshot (baseline, undo and fresh comparisons).
    pub full_probes: usize,
    /// The longest one request may take (oracle a).
    pub budget: Duration,
    /// A request running this long ends the process; `None` leaves hangs to the caller.
    pub hang: Option<Duration>,
    /// Check oracle b. Off, only the oracles that need no edit mapping run (a, c, d, e).
    pub outside: bool,
    /// Stop after this many failures.
    pub max_failures: usize,
    /// Stop editing once the session has run this long, ending it without further checks. A
    /// document that loads a large library costs several milliseconds per version, so a long
    /// run of edits could otherwise outlast whatever limit the caller runs under.
    pub deadline: Option<Duration>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scenarios: 6,
            diff_every: 7,
            probes: 12,
            full_probes: 120,
            budget: Duration::from_secs(20),
            hang: Some(Duration::from_secs(120)),
            outside: true,
            max_failures: 1,
            deadline: None,
        }
    }
}

/// What to open: configuration documents first (never edited), then the documents to edit; the
/// first of those is the one whose imports and loads the others normally are.
pub struct Input<'a> {
    pub setup: Vec<(DocumentUri, String)>,
    pub docs: Vec<(DocumentUri, String)>,
    pub environment: &'a dyn Fn() -> Environment,
}

#[derive(Default, Debug)]
pub struct Report {
    pub versions: usize,
    pub fresh_comparisons: usize,
    pub baseline_comparisons: usize,
    pub outside_comparisons: usize,
    pub failures: Vec<String>,
    /// For a failure of oracle d: the document's index and what the session went through.
    pub histories: Vec<(usize, History)>,
}

/// One text sent to the server, and the requests made while it was current.
#[derive(Clone, Debug)]
pub struct Step {
    pub text: String,
    pub queries: Vec<Query>,
}

#[derive(Clone, Debug, Default)]
pub struct History {
    /// Requests made on the original text.
    pub initial: Vec<Query>,
    pub steps: Vec<Step>,
}


/// Run one typing session over `input`, deciding with `choices`.
pub fn run(input: &Input<'_>, config: Config, choices: impl Choices) -> Report {
    // The baseline must be reproducible, or every later comparison is meaningless.
    let rig = match Rig::new(input, config, choices) {
        Ok(rig) => rig,
        Err(e) => {
            return Report {
                failures: vec![format!("could not open the documents: {e}")],
                ..Report::default()
            };
        }
    };
    let texts: Vec<String> = input.docs.iter().map(|(_, t)| t.clone()).collect();
    let mut report = Report::default();
    if let Ok(fresh) = Rig::<Seeded>::open_session(input, &texts) {
        let mut clock = Clock::new(rig.config.budget, None);
        for d in &rig.docs {
            let cap = rig.config.full_probes;
            Snapshot::take(&mut clock, &fresh, &d.uri, &d.orig, cap);
            let again = Snapshot::take(&mut clock, &fresh, &d.uri, &d.orig, cap);
            if let Some(message) = snapshot_difference(&d.baseline, &again) {
                report.failures.push(format!(
                    "baseline of {} is not reproducible: {message}",
                    d.uri.as_str()
                ));
                return report;
            }
        }
    }
    rig.run()
}

/// Send the steps of `history` to document `doc` one after another, each as the smallest
/// incremental change from the text before it, making the requests that were made in between, and
/// compare with a fresh session. Returns the difference, if there is one.
pub fn replay_difference(
    input: &Input<'_>,
    doc: usize,
    history: &History,
    config: &Config,
) -> Option<String> {
    let texts: Vec<String> = input.docs.iter().map(|(_, t)| t.clone()).collect();
    let mut session = Rig::<Seeded>::open_session(input, &texts).ok()?;
    let mut clock = Clock::new(config.budget, None);
    let uri = input.docs[doc].0.clone();
    let mut current = texts[doc].clone();
    let mut texts = texts;
    for q in &history.initial {
        q.execute(&session);
    }
    for (n, step) in history.steps.iter().enumerate() {
        let next = &step.text;
        let mut start = current
            .bytes()
            .zip(next.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while !current.is_char_boundary(start) || !next.is_char_boundary(start) {
            start -= 1;
        }
        let suffix = current[start..]
            .bytes()
            .rev()
            .zip(next[start..].bytes().rev())
            .take_while(|(a, b)| a == b)
            .count();
        let old_end = current.ceil_char_boundary(current.len() - suffix);
        let new_end = next.ceil_char_boundary(next.len() - suffix);
        let change = TextChange {
            range: Some(Range {
                start: pos_of(&current, start),
                end: pos_of(&current, old_end),
            }),
            range_length: None,
            text: next[start..new_end].to_string(),
        };
        session.change(&uri, n as i32 + 2, &[change]).ok()?;
        current.clone_from(next);
        for q in &step.queries {
            q.execute(&session);
        }
    }
    texts[doc].clone_from(&current);
    let fresh = Rig::<Seeded>::open_session(input, &texts).ok()?;
    for q in history.steps.last().map_or(&[][..], |s| &s.queries) {
        q.execute(&fresh);
    }
    let cap = config.full_probes;
    let now = Snapshot::take(&mut clock, &session, &uri, &current, cap);
    let reference = Snapshot::take(&mut clock, &fresh, &uri, &current, cap);
    if stale_damage(&now, &current) {
        return Some("reports an incomplete lexical edit for text that lexes".into());
    }
    if omits_literal(&now) || any_omits_literal(&session, input.docs.iter().map(|(u, _)| u)) {
        return None;
    }
    snapshot_difference(&reference, &now)
}

/// A shorter history (the last step always stays) that still makes [`replay_difference`] find a
/// difference: delta debugging over the steps. `None` when replaying the whole history does not
/// show the difference.
pub fn minimise_history(
    input: &Input<'_>,
    doc: usize,
    history: &History,
    config: &Config,
) -> Option<History> {
    let with = |steps: &[Step]| History {
        initial: history.initial.clone(),
        steps: steps.to_vec(),
    };
    let fails = |steps: &[Step]| replay_difference(input, doc, &with(steps), config).is_some();
    let (last, rest) = history.steps.split_last()?;
    let join = |rest: &[Step]| {
        let mut all = rest.to_vec();
        all.push(last.clone());
        all
    };
    if !fails(&join(rest)) {
        return None;
    }
    let mut rest = rest.to_vec();
    let mut chunks = 2;
    while rest.len() >= 2 {
        let size = rest.len().div_ceil(chunks);
        let mut reduced = false;
        for from in (0..rest.len()).step_by(size) {
            let mut candidate = rest.clone();
            candidate.drain(from..(from + size).min(rest.len()));
            if fails(&join(&candidate)) {
                rest = candidate;
                chunks = (chunks - 1).max(2);
                reduced = true;
                break;
            }
        }
        if !reduced {
            if chunks >= rest.len() {
                break;
            }
            chunks = (chunks * 2).min(rest.len());
        }
    }
    let mut i = 0;
    while i < rest.len() {
        let mut candidate = rest.clone();
        candidate.remove(i);
        if fails(&join(&candidate)) {
            rest = candidate;
        } else {
            i += 1;
        }
    }
    Some(with(&join(&rest)))
}
