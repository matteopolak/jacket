//! Running a test program and reading what it reports.
//!
//! A test program started with `--run NAME...` (the Testing module's `run_tests`) writes records
//! to standard error: `\x02`, the kind, each field after `\x1f`, then `\x03`. It also writes the
//! `start` record to standard output, so the output that follows can be given to its test.
//! Everything else on either stream is the tests' own output.
//!
//! One process runs one or more tests in order. It may stop early (a crash, `exit`, or a kill
//! after a timeout); [`run`] says which tests it finished, and the caller starts another process
//! for the rest.
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

/// A program to start.
#[derive(Clone, Debug)]
pub struct Invocation {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub cwd: PathBuf,
}

/// One failed check, as the test program reported it.
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    /// The test or subtest it failed in (`table/two`).
    pub test: String,
    pub file: String,
    pub line: u32,
    pub message: String,
}

/// A subtest's result.
#[derive(Clone, Debug, PartialEq)]
pub struct Subtest {
    pub name: String,
    pub status: String,
    pub duration: Duration,
}

/// What one test did in a process.
#[derive(Clone, Debug, Default)]
pub struct Raw {
    pub name: String,
    /// `pass`, `fail` or `skip` from its `end` record; `None` when the process stopped first.
    pub end: Option<String>,
    pub failures: Vec<Failure>,
    pub skip_reason: Option<String>,
    pub subtests: Vec<Subtest>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub duration: Duration,
}

/// How the process ended.
#[derive(Clone, Debug, PartialEq)]
pub enum Ending {
    Exited(i32),
    Signal(i32),
    /// Killed after the running test passed its timeout.
    TimedOut,
    /// Could not be started.
    NotStarted(String),
}

/// A process's run: the tests it started, in order, and how it ended. The last test may have no
/// `end` record: it is the one the process stopped in.
#[derive(Debug)]
pub struct Outcome {
    pub tests: Vec<Raw>,
    pub ending: Ending,
    /// Names asked for that the program does not have.
    pub missing: Vec<String>,
    /// Output before the first test started (all of a program that writes no records).
    pub preamble_stdout: Vec<u8>,
    pub preamble: Vec<u8>,
}

/// What a running process tells the scheduler as it happens.
pub enum Live<'a> {
    Started(&'a str),
    Slow(&'a str, Duration),
    /// Output to show as it comes (`--no-capture`).
    Output(&'a [u8], bool),
}

/// Limits of one process: the running test's timeout and when it counts as slow.
pub struct Limits<'a> {
    pub timeout: &'a dyn Fn(&str) -> Option<Duration>,
    pub slow: Option<Duration>,
    pub stream: bool,
}

/// One piece of a stream: text, or a record's fields (the kind first).
#[derive(Debug, PartialEq)]
pub enum Piece {
    Text(Vec<u8>),
    Record(Vec<String>),
}

/// Splits a stream into text and records, across reads.
#[derive(Default)]
pub struct Demux {
    pending: Vec<u8>,
    in_record: bool,
}

/// A record longer than this is not one: its bytes are text.
const MAX_RECORD: usize = 1 << 20;

impl Demux {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Piece> {
        let mut pieces = Vec::new();
        let mut text = Vec::new();
        for &b in bytes {
            if self.in_record {
                if b == 0x03 {
                    self.in_record = false;
                    let body = String::from_utf8_lossy(&self.pending).into_owned();
                    self.pending.clear();
                    if !text.is_empty() {
                        pieces.push(Piece::Text(std::mem::take(&mut text)));
                    }
                    pieces.push(Piece::Record(
                        body.split('\x1f').map(String::from).collect(),
                    ));
                } else if b == 0x02 || self.pending.len() > MAX_RECORD {
                    // Not a record after all.
                    text.push(0x02);
                    text.append(&mut self.pending);
                    self.in_record = b == 0x02;
                    if b != 0x02 {
                        text.push(b);
                    }
                } else {
                    self.pending.push(b);
                }
            } else if b == 0x02 {
                self.in_record = true;
            } else {
                text.push(b);
            }
        }
        if !text.is_empty() {
            pieces.push(Piece::Text(text));
        }
        pieces
    }

    /// What is left at the end of the stream.
    pub fn finish(&mut self) -> Vec<u8> {
        if !self.in_record {
            return Vec::new();
        }
        self.in_record = false;
        let mut text = vec![0x02];
        text.append(&mut self.pending);
        text
    }
}

enum Chunk {
    Out(Vec<u8>),
    Err(Vec<u8>),
    Closed,
}

fn reader(mut stream: impl Read + Send + 'static, out: bool, tx: mpsc::Sender<Chunk>) {
    std::thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            match stream.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    let bytes = buffer[..n].to_vec();
                    let chunk = if out {
                        Chunk::Out(bytes)
                    } else {
                        Chunk::Err(bytes)
                    };
                    if tx.send(chunk).is_err() {
                        return;
                    }
                }
            }
        }
        let _ = tx.send(Chunk::Closed);
    });
}

/// Stops `child` and whatever it started.
pub fn kill_tree(child: &mut Child) {
    let pid = child.id();
    #[cfg(unix)]
    {
        // The child leads its own process group (see `spawn`).
        let _ = Command::new("kill")
            .args(kill_group_args(pid))
            .stderr(Stdio::null())
            .status();
    }
    #[cfg(windows)]
    {
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

/// `kill` arguments that kill process group `pid`. The `--` matters: without it procps `kill`
/// reads `-1234` as an option, keeps only its first digit and signals pid -1, which is every
/// process the user owns.
#[cfg(unix)]
fn kill_group_args(pid: u32) -> [String; 4] {
    ["-s".into(), "KILL".into(), "--".into(), format!("-{pid}")]
}

fn spawn(invocation: &Invocation) -> std::io::Result<Child> {
    let mut command = Command::new(&invocation.program);
    command
        .args(&invocation.args)
        .current_dir(&invocation.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // Its own group, so a timeout kills the programs it started too.
        command.process_group(0);
    }
    command.spawn()
}

/// The name of the top-level test a (sub)test name belongs to.
pub fn top_name(name: &str) -> &str {
    name.split('/').next().unwrap_or(name)
}

/// Runs `invocation` to its end, reading its records.
pub fn run(invocation: &Invocation, limits: &Limits, live: &mut dyn FnMut(Live)) -> Outcome {
    let mut outcome = Outcome {
        tests: Vec::new(),
        ending: Ending::Exited(0),
        missing: Vec::new(),
        preamble_stdout: Vec::new(),
        preamble: Vec::new(),
    };
    let mut child = match spawn(invocation) {
        Ok(child) => child,
        Err(error) => {
            outcome.ending = Ending::NotStarted(format!(
                "could not start `{}`: {error}",
                invocation.program.display()
            ));
            return outcome;
        }
    };
    let (tx, rx) = mpsc::channel();
    reader(child.stdout.take().expect("piped"), true, tx.clone());
    reader(child.stderr.take().expect("piped"), false, tx);
    let mut state = State {
        last_end: Some(Instant::now()),
        ..State::default()
    };
    let mut open = 2;
    let mut timed_out = false;
    while open > 0 {
        let now = Instant::now();
        let (deadline, slow_at) =
            state
                .running(&outcome)
                .map_or((None, None), |(name, started)| {
                    let deadline = (limits.timeout)(&name).map(|t| started + t);
                    let slow = limits
                        .slow
                        .filter(|_| !state.reported_slow)
                        .map(|s| started + s);
                    (deadline, slow)
                });
        let wake = [deadline, slow_at]
            .into_iter()
            .flatten()
            .min()
            .map_or(Duration::from_millis(500), |at| {
                at.saturating_duration_since(now)
            });
        match rx.recv_timeout(wake) {
            Ok(Chunk::Closed) => open -= 1,
            Ok(Chunk::Out(bytes)) => state.stdout(&bytes, &mut outcome, limits.stream, live),
            Ok(Chunk::Err(bytes)) => state.stderr(&bytes, &mut outcome, limits.stream, live),
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
        let now = Instant::now();
        if let Some((name, started)) = state.running(&outcome) {
            if deadline.is_some_and(|d| now >= d) {
                kill_tree(&mut child);
                timed_out = true;
                // Drain what is left; the pipes close once the processes are gone.
                while open > 0 {
                    match rx.recv_timeout(Duration::from_secs(2)) {
                        Ok(Chunk::Closed) => open -= 1,
                        Ok(Chunk::Out(bytes)) => state.stdout(&bytes, &mut outcome, false, live),
                        Ok(Chunk::Err(bytes)) => state.stderr(&bytes, &mut outcome, false, live),
                        Err(_) => break,
                    }
                }
                break;
            }
            if slow_at.is_some_and(|s| now >= s) && !state.reported_slow {
                state.reported_slow = true;
                live(Live::Slow(&name, now - started));
            }
        }
    }
    let status = child.wait();
    state.finish(&mut outcome);
    outcome.ending = if timed_out {
        Ending::TimedOut
    } else {
        match status {
            Ok(status) => ending_of(status),
            Err(error) => Ending::NotStarted(error.to_string()),
        }
    };
    let end = Instant::now();
    if let Some(last) = outcome.tests.last_mut()
        && last.end.is_none()
        && let Some(started) = state.started
    {
        last.duration = end - started;
    }
    outcome
}

fn ending_of(status: std::process::ExitStatus) -> Ending {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return Ending::Signal(signal);
        }
    }
    Ending::Exited(status.code().unwrap_or(-1))
}

/// Who the output belongs to while the process runs.
#[derive(Default)]
struct State {
    out: Demux,
    err: Demux,
    /// The test standard output is written for, by its marks.
    out_test: Option<usize>,
    /// The test running, by its records.
    current: Option<usize>,
    started: Option<Instant>,
    subtest_started: Vec<(String, Instant)>,
    reported_slow: bool,
    /// When the last test ended; at first, when the process started.
    last_end: Option<Instant>,
}

impl State {
    /// The test running and when it started.
    fn running(&self, outcome: &Outcome) -> Option<(String, Instant)> {
        let test = outcome.tests.get(self.current?)?;
        Some((test.name.clone(), self.started?))
    }

    fn give(outcome: &mut Outcome, test: Option<usize>, bytes: &[u8], out: bool) {
        match test.and_then(|i| outcome.tests.get_mut(i)) {
            Some(raw) if out => raw.stdout.extend_from_slice(bytes),
            Some(raw) => raw.stderr.extend_from_slice(bytes),
            None if out => outcome.preamble_stdout.extend_from_slice(bytes),
            None => outcome.preamble.extend_from_slice(bytes),
        }
    }

    fn index_of(outcome: &Outcome, name: &str) -> Option<usize> {
        let top = top_name(name);
        outcome.tests.iter().rposition(|t| t.name == top)
    }

    fn stdout(
        &mut self,
        bytes: &[u8],
        outcome: &mut Outcome,
        stream: bool,
        live: &mut dyn FnMut(Live),
    ) {
        for piece in self.out.feed(bytes) {
            match piece {
                Piece::Text(text) => {
                    if stream {
                        live(Live::Output(&text, true));
                    }
                    Self::give(outcome, self.out_test, &text, true);
                }
                Piece::Record(fields) if fields.len() == 2 && fields[0] == "start" => {
                    self.out_test = Self::index_of(outcome, &fields[1]).or(self.out_test);
                }
                Piece::Record(_) => {}
            }
        }
    }

    fn stderr(
        &mut self,
        bytes: &[u8],
        outcome: &mut Outcome,
        stream: bool,
        live: &mut dyn FnMut(Live),
    ) {
        for piece in self.err.feed(bytes) {
            match piece {
                Piece::Text(text) => {
                    if stream {
                        live(Live::Output(&text, false));
                    }
                    Self::give(outcome, self.current, &text, false);
                }
                Piece::Record(fields) => self.record(&fields, outcome, live),
            }
        }
    }

    fn record(&mut self, fields: &[String], outcome: &mut Outcome, live: &mut dyn FnMut(Live)) {
        let field = |i: usize| fields.get(i).cloned().unwrap_or_default();
        let now = Instant::now();
        match fields[0].as_str() {
            "start" if !field(1).contains('/') => {
                outcome.tests.push(Raw {
                    name: field(1),
                    ..Raw::default()
                });
                self.current = Some(outcome.tests.len() - 1);
                // A test's time runs from the end of the one before (or the process's start):
                // its records come in quick succession, often in one read.
                self.started = Some(self.last_end.unwrap_or(now));
                self.reported_slow = false;
                if outcome.tests.len() == 1 {
                    outcome.tests[0].stderr = std::mem::take(&mut outcome.preamble);
                }
                live(Live::Started(&fields[1]));
            }
            "start" => self.subtest_started.push((field(1), now)),
            "end" => {
                let name = field(1);
                let status = field(2);
                if name.contains('/') {
                    let started = self
                        .subtest_started
                        .iter()
                        .rposition(|(n, _)| *n == name)
                        .map(|i| self.subtest_started.remove(i).1);
                    if let Some(i) = Self::index_of(outcome, &name) {
                        outcome.tests[i].subtests.push(Subtest {
                            name,
                            status,
                            duration: started.map_or(Duration::ZERO, |s| now - s),
                        });
                    }
                } else if let Some(i) = Self::index_of(outcome, &name) {
                    outcome.tests[i].end = Some(status);
                    if let Some(started) = self.started {
                        outcome.tests[i].duration = now - started;
                    }
                    self.current = None;
                    self.started = None;
                    self.last_end = Some(now);
                }
            }
            "fail" => {
                let name = field(1);
                if let Some(i) = Self::index_of(outcome, &name) {
                    outcome.tests[i].failures.push(Failure {
                        test: name,
                        file: field(2),
                        line: field(3).parse().unwrap_or(0),
                        message: field(4),
                    });
                }
            }
            "skip" => {
                if let Some(i) = Self::index_of(outcome, &field(1)) {
                    let reason = field(2);
                    outcome.tests[i].skip_reason = Some(reason);
                }
            }
            "missing" => outcome.missing.push(field(1)),
            _ => {}
        }
    }

    fn finish(&mut self, outcome: &mut Outcome) {
        let out = self.out.finish();
        Self::give(outcome, self.out_test, &out, true);
        // Standard output before the first mark: the first test's, when there is one.
        if let Some(first) = outcome.tests.first_mut() {
            let mut stdout = std::mem::take(&mut outcome.preamble_stdout);
            stdout.append(&mut first.stdout);
            first.stdout = stdout;
        }
        let err = self.err.finish();
        Self::give(
            outcome,
            self.current.or(outcome.tests.len().checked_sub(1)),
            &err,
            false,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_split_across_reads() {
        let mut demux = Demux::default();
        let mut pieces = demux.feed(b"hi\x02start\x1fa");
        pieces.extend(demux.feed(b"dds\x03 there\x02x"));
        assert_eq!(
            pieces,
            [
                Piece::Text(b"hi".to_vec()),
                Piece::Record(vec!["start".into(), "adds".into()]),
                Piece::Text(b" there".to_vec()),
            ]
        );
        assert_eq!(demux.finish(), b"\x02x");
    }

    #[test]
    fn a_stray_start_byte_is_text() {
        let mut demux = Demux::default();
        let pieces = demux.feed(b"a\x02b\x02end\x1fx\x1fpass\x03");
        assert_eq!(
            pieces,
            [
                Piece::Text(b"a\x02b".to_vec()),
                Piece::Record(vec!["end".into(), "x".into(), "pass".into()]),
            ]
        );
    }
}

#[cfg(all(test, unix))]
mod process_tests {
    use super::*;

    fn script(body: &str) -> Invocation {
        Invocation {
            program: PathBuf::from("/bin/sh"),
            args: vec!["-c".into(), body.into()],
            cwd: std::env::temp_dir(),
        }
    }

    fn no_limit(_: &str) -> Option<Duration> {
        None
    }

    #[test]
    fn records_become_tests() {
        let limits = Limits {
            timeout: &no_limit,
            slow: None,
            stream: false,
        };
        let body = r"printf '\002start\037a\003' >&2; printf '\002start\037a\003out'; printf 'err\002fail\037a\037/f.jai\0377\037bad\003\002end\037a\037fail\003' >&2; printf '\002start\037b\003' >&2; exit 3";
        let outcome = run(&script(body), &limits, &mut |_| {});
        assert_eq!(outcome.tests.len(), 2, "{outcome:?}");
        assert_eq!(outcome.tests[0].end.as_deref(), Some("fail"));
        assert_eq!(outcome.tests[0].stdout, b"out");
        assert_eq!(outcome.tests[0].stderr, b"err");
        assert_eq!(outcome.tests[0].failures[0].line, 7);
        assert_eq!(outcome.tests[1].end, None);
        assert_eq!(outcome.ending, Ending::Exited(3));
    }

    #[test]
    fn the_process_group_is_named_after_the_end_of_options() {
        assert_eq!(kill_group_args(1234), ["-s", "KILL", "--", "-1234"]);
    }

    #[test]
    fn a_timeout_kills_the_process_and_what_it_started() {
        let sleeper = std::env::temp_dir().join(format!("jacket-sleeper-{}", std::process::id()));
        let second = |_: &str| Some(Duration::from_millis(300));
        let limits = Limits {
            timeout: &second,
            slow: None,
            stream: false,
        };
        let started = Instant::now();
        let outcome = run(
            &script(&format!(
                r"printf '\002start\037a\003' >&2; sleep 30 & echo $! > {sleeper}; wait",
                sleeper = sleeper.display()
            )),
            &limits,
            &mut |_| {},
        );
        assert_eq!(outcome.ending, Ending::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(10));
        // The grandchild is gone too, and this test process is still here to check.
        let pid = std::fs::read_to_string(&sleeper).unwrap();
        let _ = std::fs::remove_file(&sleeper);
        let alive = Command::new("kill")
            .args(["-0", pid.trim()])
            .status()
            .unwrap();
        assert!(
            !alive.success(),
            "the sleep the test started is still running"
        );
    }
}
