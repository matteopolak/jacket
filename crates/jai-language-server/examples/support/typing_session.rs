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

// ---------------------------------------------------------------------------------------------
// Text helpers.

fn pos_of(text: &str, byte: usize) -> Position {
    let before = &text[..byte];
    Position {
        line: before.bytes().filter(|&b| b == b'\n').count() as u32,
        character: before
            .rsplit('\n')
            .next()
            .map_or(0, |line| line.encode_utf16().count() as u32),
    }
}

/// The byte offset of an in-range position, or `None`.
fn byte_of(text: &str, position: Position) -> Option<usize> {
    let mut start = 0;
    for _ in 0..position.line {
        start += text[start..].find('\n')? + 1;
    }
    let line = &text[start..];
    let line = &line[..line.find('\n').unwrap_or(line.len())];
    let mut units = 0;
    for (byte, ch) in line.char_indices() {
        if units == position.character {
            return Some(start + byte);
        }
        units += ch.len_utf16() as u32;
    }
    (units == position.character).then_some(start + line.len())
}

/// Offsets of identifier starts (not inside a longer word or number).
fn identifier_starts(text: &str) -> Vec<usize> {
    let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let bytes = text.as_bytes();
    (0..bytes.len())
        .filter(|&i| {
            (bytes[i].is_ascii_alphabetic() || bytes[i] == b'_') && (i == 0 || !word(bytes[i - 1]))
        })
        .collect()
}

/// Whether brackets, strings and block comments in `region` all close inside it. A string that
/// runs to the end of a line counts as unclosed, and so does a here-string.
fn balanced(region: &str) -> bool {
    if region.contains("#string") {
        return false;
    }
    let bytes = region.as_bytes();
    let (mut depth, mut comment, mut i) = (0i32, 0usize, 0);
    while i < bytes.len() {
        let two = bytes.get(i..i + 2).unwrap_or_default();
        match bytes[i] {
            _ if comment > 0 && two == b"*/" => {
                comment -= 1;
                i += 1;
            }
            _ if comment > 0 && two == b"/*" => {
                comment += 1;
                i += 1;
            }
            _ if comment > 0 => {}
            b'/' if two == b"/*" => {
                comment += 1;
                i += 1;
            }
            b'/' if two == b"//" => {
                i += region[i..].find('\n').unwrap_or(bytes.len() - i);
                continue;
            }
            b'"' => match string_end(bytes, i) {
                Some(end) => i = end,
                None => return false,
            },
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return false;
        }
        i += 1;
    }
    depth == 0 && comment == 0
}

/// The offset of the quote closing the string opened at `open`, if it closes on the same line.
fn string_end(bytes: &[u8], open: usize) -> Option<usize> {
    let mut i = open + 1;
    while let Some(&b) = bytes.get(i) {
        match b {
            b'\\' => i += 1,
            b'"' => return Some(i),
            b'\n' => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

// ---------------------------------------------------------------------------------------------
// Time budget and watchdog.

struct Clock {
    budget: Duration,
    started: Arc<AtomicU64>,
    label: Arc<Mutex<String>>,
    stop: Arc<AtomicBool>,
    epoch: Instant,
    slow: Vec<String>,
    /// Whether to keep the queries made, for replaying a failure.
    recording: bool,
    log: Vec<Query>,
}

impl Clock {
    fn new(budget: Duration, hang: Option<Duration>) -> Self {
        let clock = Self {
            budget,
            started: Arc::new(AtomicU64::new(0)),
            label: Arc::new(Mutex::new(String::new())),
            stop: Arc::new(AtomicBool::new(false)),
            epoch: Instant::now(),
            slow: Vec::new(),
            recording: false,
            log: Vec::new(),
        };
        if let Some(hang) = hang {
            let (started, label, stop, epoch) = (
                clock.started.clone(),
                clock.label.clone(),
                clock.stop.clone(),
                clock.epoch,
            );
            std::thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    std::thread::sleep(Duration::from_millis(200));
                    let began = started.load(Ordering::Relaxed);
                    let now = epoch.elapsed().as_millis() as u64 + 1;
                    if began != 0 && now.saturating_sub(began) > hang.as_millis() as u64 {
                        let what = label.lock().map(|l| l.clone()).unwrap_or_default();
                        eprintln!("typing session hung for {hang:?} in {what}");
                        std::process::exit(112);
                    }
                }
            });
        }
        clock
    }

    fn run<R>(&mut self, query: Query, f: impl FnOnce() -> R) -> R {
        if let Ok(mut l) = self.label.lock() {
            *l = format!("{query:?}");
        }
        if self.recording {
            self.log.push(query);
        }
        self.started.store(
            self.epoch.elapsed().as_millis() as u64 + 1,
            Ordering::Relaxed,
        );
        let begin = Instant::now();
        let out = f();
        self.started.store(0, Ordering::Relaxed);
        if begin.elapsed() > self.budget {
            let what = self.label.lock().map(|l| l.clone()).unwrap_or_default();
            self.slow.push(format!("{what} took {:?}", begin.elapsed()));
        }
        out
    }
}

impl Drop for Clock {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
    }
}

// ---------------------------------------------------------------------------------------------
// Queries.

/// A request made to a session, kept so that a failing run can be replayed with the same
/// requests between the same edits (what the session cached depends on them).
#[derive(Clone, Debug)]
pub enum Query {
    Diagnostics(DocumentUri),
    Tokens(DocumentUri),
    Hover(DocumentUri, Position),
    Definition(DocumentUri, Position),
    Completion(DocumentUri, Position),
    SignatureHelp(DocumentUri, Position),
}

impl Query {
    /// Make the request and drop the answer.
    pub fn execute(&self, session: &Session) {
        match self {
            Self::Diagnostics(u) => drop(session.diagnostics(u)),
            Self::Tokens(u) => drop(session.semantic_tokens(u)),
            Self::Hover(u, p) => drop(session.hover(u, *p)),
            Self::Definition(u, p) => drop(session.definition(u, *p)),
            Self::Completion(u, p) => drop(session.completion(u, *p)),
            Self::SignatureHelp(u, p) => drop(session.signature_help(u, *p)),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Snapshots.

type Answer<T> = Result<T, String>;

struct Probe {
    byte: usize,
    position: Position,
    definition: Answer<Vec<Location>>,
    hover: Answer<Option<Hover>>,
}

struct Snapshot {
    diagnostics: Answer<Vec<Diagnostic>>,
    tokens: Answer<Vec<SemanticToken>>,
    probes: Vec<Probe>,
}

fn answer<T>(result: Result<T, Error>) -> Answer<T> {
    result.map_err(|e| format!("{e:?}"))
}

fn sorted_answer<T: std::fmt::Debug>(a: &Answer<Vec<T>>) -> Answer<Vec<String>> {
    a.as_ref()
        .map(|items| sorted_debug(items))
        .map_err(Clone::clone)
}

fn sorted_debug<T: std::fmt::Debug>(items: &[T]) -> Vec<String> {
    let mut keys: Vec<_> = items.iter().map(|i| format!("{i:?}")).collect();
    keys.sort();
    keys
}

/// Identifier offsets, thinned to at most `cap` by a fixed stride.
fn sample_identifiers(text: &str, cap: usize) -> Vec<usize> {
    let skipped: Vec<(usize, usize)> = here_strings(text)
        .into_iter()
        .chain(conditional_blocks(text))
        .collect();
    let all: Vec<usize> = identifier_starts(text)
        .into_iter()
        .filter(|&i| !skipped.iter().any(|&(a, b)| a <= i && i < b))
        .collect();
    let stride = all.len().div_ceil(cap.max(1)).max(1);
    all.into_iter().step_by(stride).collect()
}

/// Byte ranges of `#if` blocks with their `else` branches. Which branch is live depends on the
/// target, and code another target would check is answered from a compile for that target, so
/// the answer there is not the one from the compile the checker compares against.
fn conditional_blocks(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut offset = 0;
    let mut open: Option<(usize, usize)> = None;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        match open {
            None if trimmed.starts_with("#if ") => {
                if trimmed.trim_end().ends_with('{') {
                    open = Some((offset, indent));
                } else {
                    out.push((offset, offset + line.len()));
                }
            }
            Some((start, at))
                if indent == at
                    && trimmed.starts_with('}')
                    && !trimmed.starts_with("} else")
                    && !trimmed.starts_with("}else") =>
            {
                out.push((start, offset + line.len()));
                open = None;
            }
            _ => {}
        }
        offset += line.len();
    }
    if let Some((start, _)) = open {
        out.push((start, text.len()));
    }
    out
}

fn probe_at(
    clock: &mut Clock,
    session: &Session,
    uri: &DocumentUri,
    text: &str,
    byte: usize,
) -> Probe {
    let position = pos_of(text, byte);
    let definition = clock.run(Query::Definition(uri.clone(), position), || {
        answer(session.definition(uri, position))
    });
    let hover = clock.run(Query::Hover(uri.clone(), position), || {
        answer(session.hover(uri, position))
    });
    Probe {
        byte,
        position,
        definition,
        hover,
    }
}

impl Snapshot {
    fn take(
        clock: &mut Clock,
        session: &Session,
        uri: &DocumentUri,
        text: &str,
        cap: usize,
    ) -> Self {
        let diagnostics = clock.run(Query::Diagnostics(uri.clone()), || {
            answer(session.diagnostics(uri))
        });
        let tokens = clock.run(Query::Tokens(uri.clone()), || {
            answer(session.semantic_tokens(uri))
        });
        let probes = sample_identifiers(text, cap)
            .into_iter()
            .map(|byte| probe_at(clock, session, uri, text, byte))
            .collect();
        Self {
            diagnostics,
            tokens,
            probes,
        }
    }
}

/// The first difference between two snapshots of the same text, if any.
fn snapshot_difference(a: &Snapshot, b: &Snapshot) -> Option<String> {
    if sorted_answer(&a.diagnostics) != sorted_answer(&b.diagnostics) {
        return Some(format!(
            "diagnostics differ:\n  was {:?}\n  now {:?}",
            a.diagnostics, b.diagnostics
        ));
    }
    if sorted_answer(&a.tokens) != sorted_answer(&b.tokens) {
        let (x, y) = (sorted_answer(&a.tokens), sorted_answer(&b.tokens));
        let detail = match (x, y) {
            (Ok(x), Ok(y)) => {
                let before = x.iter().filter(|t| !y.contains(t)).take(3);
                let after = y.iter().filter(|t| !x.contains(t)).take(3);
                format!(
                    "\n  only in the first: {:?}\n  only in the second: {:?}",
                    before.collect::<Vec<_>>(),
                    after.collect::<Vec<_>>()
                )
            }
            _ => String::new(),
        };
        return Some(format!("semantic tokens differ ({detail})"));
    }
    if a.probes.len() != b.probes.len() {
        return Some("different number of probes".into());
    }
    for (x, y) in a.probes.iter().zip(&b.probes) {
        if x.position != y.position {
            return Some(format!(
                "probe positions differ {:?} {:?}",
                x.position, y.position
            ));
        }
        if x.definition != y.definition {
            return Some(format!(
                "definition at {:?} differs:\n  was {:?}\n  now {:?}",
                x.position, x.definition, y.definition
            ));
        }
        if x.hover != y.hover {
            return Some(format!(
                "hover at {:?} differs:\n  was {:?}\n  now {:?}",
                x.position, x.hover, y.hover
            ));
        }
    }
    None
}

/// A diagnostic that follows from what a body does, not from where it is written. A lint about
/// what is used is raised by removing the only use of an import elsewhere, and a compile-time
/// call fails where it is made when the procedure it runs stops returning a value.
fn usage_lint(d: &Diagnostic) -> bool {
    matches!(d.code, DiagnosticCode::Lint(name) if name.starts_with("unused"))
        || d.message.starts_with("error during compile-time execution")
}

/// Whether the session reports an incomplete lexical edit: a literal or comment that stopped
/// closing is left out of its analysis, to keep the rest of the file's features while it is being
/// typed. A fresh session lexes the whole text as it is, so the two legitimately differ then.
fn omits_literal(incremental: &Snapshot) -> bool {
    let Ok(now) = &incremental.diagnostics else {
        return false;
    };
    now.iter()
        .any(|d| d.message.starts_with("Incomplete lexical edit"))
}

/// Oracle g: text that lexes without error has nothing left out.
fn stale_damage(incremental: &Snapshot, text: &str) -> bool {
    omits_literal(incremental) && jaic::lexer::lexical_regions(FileId(0), text).is_ok()
}

/// Oracle e on one document's diagnostics.
fn sanity(text: &str, diagnostics: &[Diagnostic]) -> Option<String> {
    let lines: Vec<&str> = text.split('\n').collect();
    let valid = |p: Position| {
        let Some(line) = lines.get(p.line as usize) else {
            return false;
        };
        // A character offset must fall between UTF-16 code points of the line.
        let mut units = 0;
        for ch in line.chars() {
            if units == p.character {
                return true;
            }
            units += ch.len_utf16() as u32;
        }
        units >= p.character
    };
    let mut seen = Vec::new();
    for d in diagnostics {
        if !valid(d.range.start) || !valid(d.range.end) || d.range.start > d.range.end {
            return Some(format!("diagnostic range outside the document: {d:?}"));
        }
        let key = format!("{d:?}");
        if seen.contains(&key) {
            return Some(format!("duplicate diagnostic: {d:?}"));
        }
        seen.push(key);
    }
    None
}

// ---------------------------------------------------------------------------------------------
// The rig: a session, the checker's copy of every document and the journal of edits.

/// A byte range of the document that edits stay inside: `start` and `orig_end` in the original
/// text, `end` in the current one. It starts at a line break and ends before one (or is the end of
/// the file), so text that follows it cannot be swallowed by a comment or string left open in it.
#[derive(Clone, Copy, Debug)]
struct Region {
    start: usize,
    orig_end: usize,
    end: usize,
}

struct Doc {
    uri: DocumentUri,
    orig: String,
    text: String,
    version: i32,
    region: Option<Region>,
    /// Every text the server was sent and what was asked of it after each, for replaying a failure.
    history: Vec<Step>,
    /// Procedure bodies of the original text, as regions.
    targets: Vec<Region>,
    baseline: Snapshot,
}

#[derive(Clone, Debug)]
struct Edit {
    at: usize,
    removed: String,
    inserted: String,
}

struct Rig<'a, C: Choices> {
    input: &'a Input<'a>,
    config: Config,
    choices: C,
    session: Session,
    clock: Clock,
    docs: Vec<Doc>,
    journal: Vec<(usize, Vec<Edit>)>,
    trace: Vec<String>,
    report: Report,
    counter: usize,
    scenario: usize,
    /// The document whose newest history step the logged requests belong to.
    last_step: Option<usize>,
    initial_log: Vec<Query>,
    hard_failures: usize,
}

/// Byte ranges of `#string NAME` bodies: source text held in a string (a program a metaprogram
/// compiles, say), where an edit changes what compile-time code does, not just what is typed.
fn here_strings(text: &str) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(at) = text[from..].find("#string ") {
        let start = from + at;
        let rest = &text[start + "#string ".len()..];
        let name = rest.split(|c: char| c.is_whitespace()).next().unwrap_or("");
        let body = rest.find('\n').map_or(rest.len(), |n| n + 1);
        let end = rest[body..]
            .lines()
            .scan(body, |offset, line| {
                let at = *offset;
                *offset += line.len() + 1;
                Some((at, line))
            })
            .find(|(_, line)| !name.is_empty() && line.trim() == name)
            .map_or(text.len(), |(at, _)| start + "#string ".len() + at);
        out.push((start, end));
        from = end.max(start + 1);
    }
    out
}

/// Procedure bodies: a line at column 0 holding `name :: (` and ending in `{`, up to the next line
/// that is just `}`. The region runs from the header's line break to the break before that `}`.
fn procedure_bodies(text: &str) -> Vec<Region> {
    let strings = here_strings(text);
    let mut out = Vec::new();
    let mut open: Option<usize> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        let start = offset;
        offset += line.len();
        let header = trimmed.ends_with('{')
            && !strings.iter().any(|&(a, b)| a <= start && start < b)
            && trimmed.contains(" :: (")
            && !trimmed.starts_with([' ', '\t', '#', '/']);
        match open {
            None if header && line.ends_with('\n') => open = Some(offset - 1),
            Some(from) if trimmed == "}" && start > from => {
                out.push(Region {
                    start: from,
                    orig_end: start - 1,
                    end: start - 1,
                });
                open = None;
            }
            _ => {}
        }
    }
    out
}

impl<'a, C: Choices> Rig<'a, C> {
    fn open_session(input: &Input<'_>, texts: &[String]) -> Result<Session, String> {
        let mut session = Session::with_environment(Limits::default(), (input.environment)());
        for (uri, text) in &input.setup {
            session
                .open(uri.clone(), 1, text.clone())
                .map_err(|e| format!("{e:?}"))?;
        }
        session.set_auto_import(false);
        for ((uri, _), text) in input.docs.iter().zip(texts) {
            session
                .open(uri.clone(), 1, text.clone())
                .map_err(|e| format!("{e:?}"))?;
        }
        Ok(session)
    }

    fn new(input: &'a Input<'a>, config: Config, choices: C) -> Result<Self, String> {
        let texts: Vec<String> = input.docs.iter().map(|(_, t)| t.clone()).collect();
        let session = Self::open_session(input, &texts)?;
        let mut clock = Clock::new(config.budget, config.hang);
        clock.recording = true;
        let docs = input
            .docs
            .iter()
            .map(|(uri, text)| Doc {
                uri: uri.clone(),
                orig: text.clone(),
                text: text.clone(),
                version: 1,
                region: None,
                history: Vec::new(),
                targets: procedure_bodies(text),
                baseline: Snapshot::take(&mut clock, &session, uri, text, config.full_probes),
            })
            .collect();
        // Asking again must change nothing: the second snapshot is the baseline, and a difference
        // from the first is oracle h's.
        let mut docs: Vec<Doc> = docs;
        let mut asked_twice = Vec::new();
        for d in &mut docs {
            let again = Snapshot::take(&mut clock, &session, &d.uri, &d.orig, config.full_probes);
            if let Some(message) = snapshot_difference(&d.baseline, &again) {
                asked_twice.push((d.uri.as_str().to_string(), message));
            }
            d.baseline = again;
        }
        let mut rig = Self {
            input,
            config,
            choices,
            session,
            clock,
            docs,
            journal: Vec::new(),
            trace: Vec::new(),
            report: Report::default(),
            counter: 0,
            scenario: 0,
            last_step: None,
            initial_log: Vec::new(),
            hard_failures: 0,
        };
        for (uri, message) in asked_twice {
            rig.report.failures.push(format!(
                "oracle h in {uri}: the same requests answered differently the second time:\n{message}"
            ));
        }
        Ok(rig)
    }

    /// File the requests logged so far under the version they were made on.
    fn flush_log(&mut self) {
        let queries: Vec<Query> = std::mem::take(&mut self.clock.log);
        match self.last_step {
            Some(doc) => {
                if let Some(step) = self.docs[doc].history.last_mut() {
                    step.queries.extend(queries);
                }
            }
            None => self.initial_log.extend(queries),
        }
    }

    fn failed(&self) -> bool {
        self.hard_failures >= self.config.max_failures
    }

    fn fail(&mut self, oracle: &str, doc: usize, detail: String) {
        if oracle != "h" {
            self.hard_failures += 1;
        }
        let recent: Vec<_> = self.trace.iter().rev().take(6).rev().cloned().collect();
        let text: String = self.docs[doc].text.chars().take(1500).collect();
        if oracle == "d" {
            self.flush_log();
            self.report.histories.push((
                doc,
                History {
                    initial: self.initial_log.clone(),
                    steps: self.docs[doc].history.clone(),
                },
            ));
        }
        self.report.failures.push(format!(
            "oracle {oracle} in {} (scenario {}, version {}, {} edits so far):\n{detail}\nlast edits:\n  {}\ntext:\n{text}",
            self.docs[doc].uri.as_str(),
            self.scenario,
            self.report.versions,
            self.trace.len(),
            recent.join("\n  ")
        ));
    }

    fn at_baseline(&self) -> bool {
        self.docs.iter().all(|d| d.text == d.orig)
    }

    // -- editing ---------------------------------------------------------------------------

    /// Send `edits` as one `didChange`, in order, and record them in the journal.
    fn apply(&mut self, doc: usize, edits: Vec<Edit>) {
        self.apply_inner(doc, &edits);
        self.journal.push((doc, edits));
    }

    fn apply_inner(&mut self, doc: usize, edits: &[Edit]) {
        self.flush_log();
        self.last_step = Some(doc);
        let d = &mut self.docs[doc];
        let mut changes = Vec::new();
        for e in edits {
            let end = e.at + e.removed.len();
            assert_eq!(&d.text[e.at..end], e.removed, "checker edit out of sync");
            changes.push(TextChange {
                range: Some(Range {
                    start: pos_of(&d.text, e.at),
                    end: pos_of(&d.text, end),
                }),
                range_length: None,
                text: e.inserted.clone(),
            });
            d.text.replace_range(e.at..end, &e.inserted);
            if let Some(r) = &mut d.region {
                assert!(e.at >= r.start && end <= r.end, "edit left its region");
                r.end = r.end - e.removed.len() + e.inserted.len();
            }
            self.trace.push(format!(
                "doc {doc} at {} remove {:?} insert {:?}",
                e.at, e.removed, e.inserted
            ));
        }
        d.version += 1;
        d.history.push(Step {
            text: d.text.clone(),
            queries: Vec::new(),
        });
        let (uri, version) = (d.uri.clone(), d.version);
        if let Err(e) = self.session.change(&uri, version, &changes) {
            self.fail("f", doc, format!("change rejected: {e:?}"));
        }
        self.report.versions += 1;
        self.counter += 1;
    }

    fn undo_one(&mut self) -> bool {
        let Some((doc, edits)) = self.journal.pop() else {
            return false;
        };
        let inverse: Vec<Edit> = edits
            .iter()
            .rev()
            .map(|e| Edit {
                at: e.at,
                removed: e.inserted.clone(),
                inserted: e.removed.clone(),
            })
            .collect();
        self.apply_inner(doc, &inverse);
        self.forget_regions();
        true
    }

    /// A document back at its original text has no region (the next scenario picks one).
    fn forget_regions(&mut self) {
        for d in &mut self.docs {
            if d.text == d.orig {
                d.region = None;
            }
        }
        if self.at_baseline() {
            self.journal.clear();
        }
    }

    fn insert(&mut self, doc: usize, at: usize, text: &str) {
        self.apply(
            doc,
            vec![Edit {
                at,
                removed: String::new(),
                inserted: text.into(),
            }],
        );
    }

    fn remove(&mut self, doc: usize, at: usize, len: usize) {
        let removed = self.docs[doc].text[at..at + len].to_string();
        self.apply(
            doc,
            vec![Edit {
                at,
                removed,
                inserted: String::new(),
            }],
        );
    }

    // -- checks ----------------------------------------------------------------------------

    /// Run after every version.
    fn after_edit(&mut self) {
        if self.failed() {
            return;
        }
        self.check_light();
        if self.config.diff_every > 0 && self.counter.is_multiple_of(self.config.diff_every) {
            self.check_fresh();
        }
        if self.at_baseline() {
            self.check_baseline();
        }
        for slow in std::mem::take(&mut self.clock.slow) {
            self.fail("a", 0, format!("slow request: {slow}"));
        }
    }

    fn region_text(&self, doc: usize) -> Option<&str> {
        let d = &self.docs[doc];
        d.region.map(|r| &d.text[r.start..r.end])
    }

    fn all_balanced(&self) -> bool {
        (0..self.docs.len()).all(|i| self.region_text(i).is_none_or(balanced))
    }

    /// Map a byte of the original text to the current one; `None` inside the region.
    fn map_byte(&self, doc: usize, byte: usize, is_end: bool) -> Option<usize> {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return Some(byte);
        };
        if byte < r.start || (is_end && byte == r.start) {
            Some(byte)
        } else if byte >= r.orig_end {
            Some(byte - r.orig_end + r.end)
        } else {
            None
        }
    }

    fn map_position(&self, doc: usize, p: Position, is_end: bool) -> Option<Position> {
        let d = &self.docs[doc];
        let byte = byte_of(&d.orig, p)?;
        let mapped = self.map_byte(doc, byte, is_end)?;
        Some(pos_of(&d.text, mapped))
    }

    fn map_range(&self, doc: usize, r: Range) -> Option<Range> {
        Some(Range {
            start: self.map_position(doc, r.start, false)?,
            end: self.map_position(doc, r.end, true)?,
        })
    }

    fn map_location(&self, l: &Location) -> Option<Location> {
        let Some(target) = self.docs.iter().position(|d| d.uri.as_str() == l.uri) else {
            return Some(l.clone());
        };
        Some(Location {
            uri: l.uri.clone(),
            range: self.map_range(target, l.range)?,
        })
    }

    /// Whether a range of `text` touches `lo..hi`.
    fn touches(text: &str, range: Range, lo: usize, hi: usize) -> bool {
        let (Some(a), Some(b)) = (byte_of(text, range.start), byte_of(text, range.end)) else {
            return true;
        };
        if a == b {
            return lo < a && a <= hi;
        }
        a < hi && b > lo
    }

    /// Whether a current-text range touches the region of `doc`.
    fn in_region(&self, doc: usize, range: Range) -> bool {
        let d = &self.docs[doc];
        d.region
            .is_some_and(|r| Self::touches(&d.text, range, r.start, r.end))
    }

    /// Whether a diagnostic range touches the region, the header line above it or the closing line
    /// below it: a missing return or an unused parameter is reported there, on what the body is
    /// checked against.
    fn near_region(&self, doc: usize, range: Range, original: bool) -> bool {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return false;
        };
        let (text, end) = if original {
            (&d.orig, r.orig_end)
        } else {
            (&d.text, r.end)
        };
        let line = text[..r.start].rfind('\n').map_or(0, |i| i + 1);
        // The line after the body too: the closing brace is where an unfinished statement ends.
        let after = text[end..]
            .strip_prefix('\n')
            .map_or(end, |rest| end + 1 + rest.find('\n').unwrap_or(rest.len()));
        Self::touches(text, range, line, after)
    }

    fn check_light(&mut self) {
        let texts: Vec<_> = self
            .docs
            .iter()
            .map(|d| (d.uri.clone(), d.text.clone()))
            .collect();
        for (i, (uri, text)) in texts.iter().enumerate() {
            let seen = self.session.document_text(uri).map(str::to_owned);
            if seen.as_deref() != Ok(text.as_str()) {
                self.fail(
                    "f",
                    i,
                    "the server's text differs from the edited text".into(),
                );
                return;
            }
            let diagnostics = self.clock.run(Query::Diagnostics(uri.clone()), || {
                self.session.diagnostics(uri)
            });
            match diagnostics {
                Ok(list) => {
                    if let Some(message) = sanity(text, &list) {
                        self.fail("e", i, message);
                    }
                }
                Err(e) => self.fail("e", i, format!("diagnostics failed: {e:?}")),
            }
            self.cursor_queries(i);
        }
        if self.config.outside && self.all_balanced() && !self.failed() {
            for i in 0..self.docs.len() {
                if let Some(message) = self.outside_difference(i) {
                    self.fail("b", i, message);
                    return;
                }
            }
        }
    }

    /// Requests at the edit site, only to be survived (answers there may be anything).
    fn cursor_queries(&mut self, doc: usize) {
        let d = &self.docs[doc];
        let Some(r) = d.region else {
            return;
        };
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let at = r.start + (self.counter * 7) % (r.end - r.start + 1);
        let at = text.floor_char_boundary(at);
        let p = pos_of(&text, at);
        let s = &self.session;
        for query in [
            Query::Hover(uri.clone(), p),
            Query::Definition(uri.clone(), p),
            Query::Completion(uri.clone(), p),
            Query::SignatureHelp(uri.clone(), p),
        ] {
            self.clock.run(query.clone(), || query.execute(s));
        }
    }

    /// Oracle b for one document: baseline answers outside the region, mapped through the edits.
    fn outside_difference(&mut self, doc: usize) -> Option<String> {
        self.report.outside_comparisons += 1;
        let d = &self.docs[doc];
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let base = &d.baseline;
        // A rotating window of the baseline probes.
        let n = base.probes.len();
        let take = self.config.probes.min(n);
        let from = if n == 0 {
            0
        } else {
            (self.counter * 5) % n
        };
        let mut found = None;
        for k in 0..take {
            let probe = &base.probes[(from + k) % n];
            let Some(byte) = self.map_byte(doc, probe.byte, false) else {
                continue;
            };
            let (Ok(old_def), Ok(old_hover)) = (&probe.definition, &probe.hover) else {
                continue;
            };
            let expected: Option<Vec<Location>> =
                old_def.iter().map(|l| self.map_location(l)).collect();
            let Some(expected) = expected else {
                continue;
            };
            let now = probe_at(&mut self.clock, &self.session, &uri, &text, byte);
            if now.definition.as_ref() != Ok(&expected) {
                found = Some(format!(
                    "definition at {:?} (was {:?}) changed:\n  baseline {:?}\n  now      {:?}",
                    now.position, probe.position, expected, now.definition
                ));
                break;
            }
            let expected_hover = old_hover.as_ref().map(|h| {
                self.map_range(doc, h.range).map(|range| Hover {
                    contents: h.contents.clone(),
                    range,
                })
            });
            let comparable = match expected_hover {
                None => Some(None),
                Some(Some(h)) => Some(Some(h)),
                Some(None) => None,
            };
            if let Some(expected_hover) = comparable
                && now.hover.as_ref() != Ok(&expected_hover)
            {
                found = Some(format!(
                    "hover at {:?} (was {:?}) changed:\n  baseline {:?}\n  now      {:?}",
                    now.position, probe.position, expected_hover, now.hover
                ));
                break;
            }
        }
        if found.is_some() {
            return found;
        }
        self.outside_lists(doc)
    }

    fn outside_lists(&mut self, doc: usize) -> Option<String> {
        let d = &self.docs[doc];
        let (uri, text) = (d.uri.clone(), d.text.clone());
        let session = &self.session;
        let diagnostics = self.clock.run(Query::Diagnostics(uri.clone()), || {
            answer(session.diagnostics(&uri))
        });
        let tokens = self.clock.run(Query::Tokens(uri.clone()), || {
            answer(session.semantic_tokens(&uri))
        });
        let d = &self.docs[doc];
        if let (Ok(now), Ok(was)) = (&diagnostics, &d.baseline.diagnostics) {
            let expected: Vec<_> = was
                .iter()
                .filter(|x| !usage_lint(x) && !self.near_region(doc, x.range, true))
                .filter_map(|x| {
                    Some(Diagnostic {
                        range: self.map_range(doc, x.range)?,
                        ..x.clone()
                    })
                })
                .collect();
            let actual: Vec<_> = now
                .iter()
                .filter(|x| !usage_lint(x) && !self.near_region(doc, x.range, false))
                .cloned()
                .collect();
            if sorted_debug(&expected) != sorted_debug(&actual) {
                return Some(format!(
                    "diagnostics outside the edited region changed:\n  baseline {expected:?}\n  now      {actual:?}"
                ));
            }
        }
        if let (Ok(now), Ok(was)) = (&tokens, &d.baseline.tokens) {
            let expected: Vec<_> = was
                .iter()
                .filter_map(|t| {
                    Some(SemanticToken {
                        position: self.map_position(doc, t.position, false)?,
                        ..*t
                    })
                })
                .collect();
            let actual: Vec<_> = now
                .iter()
                .filter(|t| {
                    let end = Position {
                        line: t.position.line,
                        character: t.position.character + t.length,
                    };
                    !self.in_region(
                        doc,
                        Range {
                            start: t.position,
                            end,
                        },
                    )
                })
                .copied()
                .collect();
            if sorted_debug(&expected) != sorted_debug(&actual) {
                let (x, y) = (sorted_debug(&expected), sorted_debug(&actual));
                let first = x
                    .iter()
                    .find(|t| !y.contains(t))
                    .map(|t| format!("lost {t}"))
                    .or_else(|| {
                        y.iter()
                            .find(|t| !x.contains(t))
                            .map(|t| format!("gained {t}"))
                    });
                return Some(format!(
                    "semantic tokens outside the edited region changed: {first:?} ({} -> {})",
                    expected.len(),
                    actual.len()
                ));
            }
        }
        let _ = text;
        None
    }

    /// Oracle c: the original text again must give the baseline back exactly.
    fn check_baseline(&mut self) {
        if self.failed() {
            return;
        }
        self.report.baseline_comparisons += 1;
        for i in 0..self.docs.len() {
            let d = &self.docs[i];
            let (uri, text) = (d.uri.clone(), d.text.clone());
            let now = Snapshot::take(
                &mut self.clock,
                &self.session,
                &uri,
                &text,
                self.config.full_probes,
            );
            if let Some(message) = snapshot_difference(&self.docs[i].baseline, &now) {
                self.fail(
                    "c",
                    i,
                    format!("after returning to the original text: {message}"),
                );
                return;
            }
        }
    }

    /// Oracle d: a session opened on the current text answers the same.
    fn check_fresh(&mut self) {
        self.report.fresh_comparisons += 1;
        let texts: Vec<String> = self.docs.iter().map(|d| d.text.clone()).collect();
        let fresh = match Self::open_session(self.input, &texts) {
            Ok(s) => s,
            Err(e) => {
                self.fail("d", 0, format!("fresh session failed to open: {e}"));
                return;
            }
        };
        // What the facts of a broken declaration hold depends on which requests were made on this
        // text, so the fresh session is asked the same ones first: only state carried over from
        // earlier versions can then make it differ.
        for query in &self.clock.log {
            query.execute(&fresh);
        }
        for i in 0..self.docs.len() {
            let d = &self.docs[i];
            let (uri, text) = (d.uri.clone(), d.text.clone());
            let cap = self.config.full_probes;
            let now = Snapshot::take(&mut self.clock, &self.session, &uri, &text, cap);
            self.clock.recording = false;
            let reference = Snapshot::take(&mut self.clock, &fresh, &uri, &text, cap);
            self.clock.recording = true;
            if stale_damage(&now, &text) {
                self.fail(
                    "g",
                    i,
                    "reports an incomplete lexical edit for text that lexes".into(),
                );
                return;
            }
            if omits_literal(&now) {
                continue;
            }
            if let Some(message) = snapshot_difference(&reference, &now) {
                self.fail(
                    "d",
                    i,
                    format!("differs from a fresh session (was fresh, now incremental): {message}"),
                );
                return;
            }
        }
    }

    // -- scenarios -------------------------------------------------------------------------

    fn pick_doc(&mut self) -> usize {
        let n = self.docs.len();
        // Mostly the first document; the others get a third of the sessions between them.
        if n > 1 && self.choices.chance(1, 3) {
            1 + self.choices.below(n - 1)
        } else {
            0
        }
    }

    /// Give `doc` a region to edit in (a procedure body, or the end of the file).
    fn ensure_region(&mut self, doc: usize) {
        if self.docs[doc].region.is_some() {
            return;
        }
        let targets = self.docs[doc].targets.len();
        let end = self.docs[doc].orig.len();
        let region = if targets > 0 && !self.choices.chance(1, 6) {
            self.docs[doc].targets[self.choices.below(targets)]
        } else {
            Region {
                start: end,
                orig_end: end,
                end,
            }
        };
        self.docs[doc].region = Some(region);
    }

    /// Offsets in the region where a line can be opened: the ends of its lines.
    fn line_ends(&self, doc: usize) -> Vec<usize> {
        let d = &self.docs[doc];
        let r = d.region.expect("region");
        let mut out: Vec<usize> = (r.start..r.end)
            .filter(|&i| d.text.as_bytes()[i] == b'\n')
            .collect();
        out.push(r.end);
        out
    }

    fn statement(&mut self) -> String {
        const SNIPPETS: &[&str] = &[
            "__ty := 1;",
            "__ty: int = 7;",
            "__ty := 1;\n    __ty += 2;",
            "if true { __ty := 3; }",
            "for 0..3 { __ty := it; }",
            "print(\"%\\n\", 1);",
            "{\n        __ty := 1;\n    }",
            "__ty :: (a: int) -> int { return a + 1; }",
            "__ty :: struct { a: int; b: float; }",
            "__ty :: enum { A; B; }",
            "__ty := cast(float) 2 * 1.5;",
            "x := __ty.a;",
            "// a comment",
            "/* block */ __ty := 1;",
            "s := \"text\\n\";",
            "if 1 == 1 { } else { }",
            "while true { break; }",
            "__ty := (1 + 2) * 3;",
            "a: [4] int;",
            "p := *__ty;",
        ];
        let name = format!("__ty{}", self.counter);
        let mut statement = SNIPPETS[self.choices.below(SNIPPETS.len())].replace("__ty", &name);
        // Sometimes a call or use of a name the file already resolves.
        if self.choices.chance(1, 3) {
            let words: Vec<&str> = self.docs[0]
                .baseline
                .probes
                .iter()
                .filter(|p| matches!(&p.definition, Ok(l) if !l.is_empty()))
                .map(|p| {
                    let rest = &self.docs[0].orig[p.byte..];
                    let end = rest
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .unwrap_or(rest.len());
                    &rest[..end]
                })
                .collect();
            if !words.is_empty() {
                let word = words[self.choices.below(words.len())];
                statement = if self.choices.chance(1, 2) {
                    format!("{word}(1, 2);")
                } else {
                    format!("{name} := {word};")
                };
            }
        }
        format!("    {statement}")
    }

    /// Type `text` character by character at `cursor`. With `autoclose` an opening bracket or quote
    /// inserts its pair and typing the closer steps over it, as editors do. Returns the cursor.
    fn type_text(&mut self, doc: usize, mut cursor: usize, text: &str, autoclose: bool) -> usize {
        for ch in text.chars() {
            if self.failed() {
                break;
            }
            let after = self.docs[doc].text[cursor..].chars().next();
            if autoclose && matches!(ch, ')' | ']' | '}' | '"') && after == Some(ch) {
                cursor += 1;
                continue;
            }
            let closer = match ch {
                '(' => Some(')'),
                '[' => Some(']'),
                '{' => Some('}'),
                '"' => Some('"'),
                _ => None,
            };
            let mut typed = ch.to_string();
            if let Some(closer) = closer.filter(|_| autoclose) {
                typed.push(closer);
            }
            self.insert(doc, cursor, &typed);
            cursor += ch.len_utf8();
            self.after_edit();
        }
        cursor
    }

    /// Delete backwards from `cursor` down to `stop`, one character per version.
    fn backspace(&mut self, doc: usize, mut cursor: usize, stop: usize) -> usize {
        while cursor > stop && !self.failed() {
            let text = &self.docs[doc].text;
            let ch = text[..cursor].chars().next_back().expect("character");
            let len = ch.len_utf8();
            let pair = matches!(ch, '(' | '[' | '{' | '"')
                && text[cursor..].starts_with(match ch {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    _ => '"',
                });
            let (at, total) = if pair {
                (cursor - len, len + 1)
            } else {
                (cursor - len, len)
            };
            self.remove(doc, at, total);
            cursor -= len;
            self.after_edit();
        }
        cursor
    }

    fn scenario_type(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = if self.choices.chance(2, 3) {
            *ends.last().expect("end")
        } else {
            ends[self.choices.below(ends.len())]
        };
        // Open the new line first, as Enter does.
        self.insert(doc, site, "\n");
        self.after_edit();
        let statement = self.statement();
        let autoclose = self.choices.chance(1, 2);
        let cursor = self.type_text(doc, site + 1, &statement, autoclose);
        if self.choices.chance(1, 2) {
            // Backspace part of it, or all of it and the new line.
            let keep = if self.choices.chance(1, 2) {
                site + 1
            } else {
                site + 1 + (cursor - site - 1) / 2
            };
            let cursor = self.backspace(doc, cursor, keep);
            if keep == site + 1 && !self.failed() {
                self.backspace(doc, cursor, site);
            }
        }
    }

    fn scenario_retype(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let lines: Vec<(usize, usize)> = ends
            .windows(2)
            .map(|w| (w[0] + 1, w[1]))
            .filter(|&(s, e)| s < e && !self.docs[doc].text[s..e].trim().is_empty())
            .collect();
        if lines.is_empty() {
            return self.scenario_type(doc);
        }
        let (mut start, mut end) = lines[self.choices.below(lines.len())];
        if self.choices.chance(1, 2) {
            // Just one token of the line.
            let text = &self.docs[doc].text;
            let words: Vec<(usize, usize)> = identifier_starts(&text[start..end])
                .into_iter()
                .map(|i| {
                    let rest = &text[start + i..end];
                    let len = rest
                        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                        .unwrap_or(rest.len());
                    (start + i, start + i + len)
                })
                .collect();
            if !words.is_empty() {
                (start, end) = words[self.choices.below(words.len())];
            }
        }
        let original = self.docs[doc].text[start..end].to_string();
        if self.choices.chance(1, 2) {
            self.remove(doc, start, end - start);
            self.after_edit();
        } else {
            self.backspace(doc, end, start);
        }
        self.type_text(doc, start, &original, false);
    }

    fn scenario_paste(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = ends[self.choices.below(ends.len())];
        let block = if self.choices.chance(1, 2) {
            // The region's own text again.
            let d = &self.docs[doc];
            let r = d.region.expect("region");
            d.text[r.start..r.end].to_string()
        } else {
            let first = self.statement();
            let second = self.statement();
            format!("\n{first}\n{second}")
        };
        if block.is_empty() {
            return;
        }
        self.insert(doc, site, &block);
        self.after_edit();
        if self.choices.chance(1, 2) {
            self.undo_one();
            self.after_edit();
        }
    }

    fn scenario_unbalanced(&mut self, doc: usize) {
        self.ensure_region(doc);
        let ends = self.line_ends(doc);
        let site = ends[self.choices.below(ends.len())];
        let (open, close) = [
            ("{", "}"),
            ("(", ")"),
            ("[", "]"),
            ("\"", "\""),
            ("/*", "*/"),
            ("#string END\n", "\nEND"),
            ("/*", ""),
            ("\"", ""),
        ][self.choices.below(8)];
        // Open it at the end of a line, check, then close it at the end of the region.
        self.insert(doc, site, open);
        self.after_edit();
        if self.choices.chance(1, 2) {
            // Type something while it is open.
            let statement = self.statement();
            let cursor = site + open.len();
            self.type_text(doc, cursor, &format!("\n{statement}"), false);
        }
        if !close.is_empty() && !self.failed() {
            let end = self.docs[doc].region.expect("region").end;
            self.insert(doc, end, close);
            self.after_edit();
        }
    }

    fn scenario_undo(&mut self) {
        let steps = 1 + self.choices.below(self.journal.len().max(1));
        for _ in 0..steps {
            if self.failed() || !self.undo_one() {
                break;
            }
            self.after_edit();
        }
    }

    /// Revert every document the way an editor's "revert file" does: replace it whole.
    fn revert_whole(&mut self) {
        for i in 0..self.docs.len() {
            if self.docs[i].text == self.docs[i].orig {
                continue;
            }
            self.flush_log();
            self.last_step = Some(i);
            let d = &mut self.docs[i];
            d.version += 1;
            d.text = d.orig.clone();
            d.history.push(Step {
                text: d.text.clone(),
                queries: Vec::new(),
            });
            d.region = None;
            let (uri, version, text) = (d.uri.clone(), d.version, d.text.clone());
            let change = TextChange {
                range: None,
                range_length: None,
                text,
            };
            if let Err(e) = self.session.change(&uri, version, &[change]) {
                self.fail("f", i, format!("whole-document change rejected: {e:?}"));
            }
            self.trace
                .push(format!("doc {i} replaced by the original text"));
            self.report.versions += 1;
            self.counter += 1;
        }
        self.journal.clear();
        self.after_edit();
    }

    fn run_scenario(&mut self) {
        let doc = self.pick_doc();
        match self.choices.below(10) {
            0..=3 => self.scenario_type(doc),
            4 | 5 => self.scenario_retype(doc),
            6 => self.scenario_paste(doc),
            7 | 8 => self.scenario_unbalanced(doc),
            _ => self.scenario_type(doc),
        }
        // Sometimes carry on from the edited text; otherwise go back.
        match self.choices.below(4) {
            0 => {}
            1 => self.scenario_undo(),
            2 => self.revert_whole(),
            _ => {
                while !self.failed() && self.undo_one() {
                    if self.choices.chance(1, 3) {
                        self.after_edit();
                    }
                }
                self.after_edit();
            }
        }
    }

    fn run(mut self) -> Report {
        for scenario in 0..self.config.scenarios {
            if self.failed() || self.choices.exhausted() {
                break;
            }
            self.scenario = scenario;
            self.run_scenario();
        }
        if !self.failed() {
            // End where the document began, by the editor's undo.
            while self.undo_one() {
                if self.choices.chance(1, 4) {
                    self.after_edit();
                }
            }
            self.revert_whole();
            self.after_edit();
        }
        self.report
    }
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
    texts[doc] = current.clone();
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
    if omits_literal(&now) {
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
