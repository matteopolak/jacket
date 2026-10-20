//! The build status `jaic` shows on a terminal: a `Compiling` line, a status line redrawn in place
//! while the compiler works, and a `Finished` line. See `docs/tools/cli.md`.
//!
//! The compiler reports through relaxed atomics that cost one load when the status is off
//! (`enter`, `file_loaded`, `module_created`, `procedure_checked`). A ticker thread samples them
//! about 12 times a second and redraws; nothing on the compiler's side waits for it. Anything
//! that writes to the terminal while the line may be up calls `before_output` first, which
//! erases it and keeps it down for a moment so output is never mixed with it.
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};
use std::time::{Duration, Instant};

/// How the status is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    /// Nothing.
    Off,
    /// The `Compiling` and `Finished` lines, without colour or control codes.
    Plain,
    /// Those lines in colour, and a status line redrawn in place.
    Live,
}

/// Picks the mode. `JAIC_PROGRESS=off|plain|live` wins over everything but `quiet`; otherwise a
/// terminal gets `Live`, or `Plain` when it cannot or should not take control codes (`NO_COLOR`,
/// `-no_color`, `TERM=dumb`, `CI`, a Windows console without ANSI support), and anything else
/// `Off`, so logs and pipes see no change.
pub fn mode_with(
    terminal: bool,
    quiet: bool,
    no_color: bool,
    env: impl Fn(&str) -> Option<String>,
) -> Mode {
    if quiet {
        return Mode::Off;
    }
    match env("JAIC_PROGRESS").as_deref() {
        Some("off") => return Mode::Off,
        Some("plain") => return Mode::Plain,
        Some("live") => return Mode::Live,
        _ => {}
    }
    if !terminal {
        return Mode::Off;
    }
    let dumb = env("TERM").as_deref() == Some("dumb");
    let ansi_console = !cfg!(windows)
        || env("WT_SESSION").is_some()
        || env("ANSICON").is_some()
        || env("ConEmuANSI").as_deref() == Some("ON")
        || env("TERM").is_some();
    let ci = env("CI").is_some_and(|v| !v.is_empty() && v != "0" && v != "false");
    if no_color || dumb || ci || !ansi_console || env("NO_COLOR").is_some() {
        Mode::Plain
    } else {
        Mode::Live
    }
}

/// `mode_with` for this process: stderr and its environment.
pub fn mode(quiet: bool, no_color: bool) -> Mode {
    use std::io::IsTerminal;
    mode_with(std::io::stderr().is_terminal(), quiet, no_color, |key| {
        std::env::var(key).ok()
    })
}

/// What the compiler is doing, as the status line names it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Phase {
    Starting,
    Parsing,
    Checking,
    /// Compile-time code: `#run`, a constant that needed the interpreter, or a metaprogram.
    Running,
    /// IR for the output, then machine code at `-O0`.
    Generating,
    /// Machine code for an optimized build.
    Optimizing,
    Linking,
    /// The interpreted program runs and owns the terminal: no status line.
    Program,
}

impl Phase {
    const ALL: [Phase; 8] = [
        Phase::Starting,
        Phase::Parsing,
        Phase::Checking,
        Phase::Running,
        Phase::Generating,
        Phase::Optimizing,
        Phase::Linking,
        Phase::Program,
    ];

    fn verb(self) -> &'static str {
        match self {
            Phase::Starting => "Starting",
            Phase::Parsing => "Parsing",
            Phase::Checking => "Checking",
            Phase::Running => "Running",
            Phase::Generating => "Generating",
            Phase::Optimizing => "Optimizing",
            Phase::Linking => "Linking",
            Phase::Program => "Running",
        }
    }
}

/// Whether the status line is up: the counters below are kept only then.
static LIVE: AtomicBool = AtomicBool::new(false);

static PHASE: AtomicU8 = AtomicU8::new(Phase::Starting as u8);
static FILES: AtomicU32 = AtomicU32::new(0);
static MODULES: AtomicU32 = AtomicU32::new(0);
static PROCEDURES: AtomicU32 = AtomicU32::new(0);

/// Machine code is optimized, so `Generating` machine code reads `Optimizing`.
static OPTIMIZED: AtomicBool = AtomicBool::new(false);

#[inline]
fn live() -> bool {
    LIVE.load(Ordering::Relaxed)
}

/// A source file was read into the program.
#[inline]
pub fn file_loaded() {
    if live() {
        FILES.fetch_add(1, Ordering::Relaxed);
    }
}

/// A module was imported for the first time.
#[inline]
pub fn module_created() {
    if live() {
        MODULES.fetch_add(1, Ordering::Relaxed);
    }
}

/// A procedure body was checked.
#[inline]
pub fn procedure_checked() {
    if live() {
        PROCEDURES.fetch_add(1, Ordering::Relaxed);
    }
}

/// The machine code about to be generated is optimized.
pub fn set_optimized(optimized: bool) {
    OPTIMIZED.store(optimized, Ordering::Relaxed);
}

/// Puts the previous phase back when dropped.
#[must_use]
pub struct PhaseGuard(Option<u8>);

impl Drop for PhaseGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.0 {
            PHASE.store(previous, Ordering::Relaxed);
        }
    }
}

/// Enter `phase` until the guard is dropped. Phases nest: a `#run` inside checking goes back
/// to checking.
#[inline]
pub fn enter(phase: Phase) -> PhaseGuard {
    if !live() {
        return PhaseGuard(None);
    }
    let mut phase = phase;
    if phase == Phase::Generating && OPTIMIZED.load(Ordering::Relaxed) {
        phase = Phase::Optimizing;
    }
    if phase == Phase::Program {
        before_output();
    }
    PhaseGuard(Some(PHASE.swap(phase as u8, Ordering::Relaxed)))
}

/// What the status line shows, sampled from the counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Status<'a> {
    pub phase: Phase,
    pub name: &'a str,
    pub files: u32,
    pub modules: u32,
    pub procedures: u32,
    pub elapsed: Duration,
}

/// `1234567` as `1,234,567`.
fn grouped(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

fn plural(n: u32, one: &str, many: &str) -> String {
    format!(
        "{} {}",
        grouped(n),
        if n == 1 {
            one
        } else {
            many
        }
    )
}

/// The verb, right-aligned in 12 columns like cargo's, and the rest of a status line.
fn columns(verb: &str) -> String {
    format!("{verb:>12}")
}

/// The status line without colour, at most `width` characters:
/// `    Checking 412 files, 38 modules, 5,120 procedures (1.4s)`.
pub fn status_line(status: &Status, width: usize) -> String {
    let counts = || {
        let mut parts = vec![plural(status.files, "file", "files")];
        if status.modules > 0 {
            parts.push(plural(status.modules, "module", "modules"));
        }
        if status.procedures > 0 {
            parts.push(plural(status.procedures, "procedure", "procedures"));
        }
        parts.join(", ")
    };
    let detail = match status.phase {
        Phase::Starting | Phase::Linking | Phase::Program => status.name.to_string(),
        Phase::Parsing | Phase::Checking => counts(),
        Phase::Running => format!("#run, {}", counts()),
        Phase::Generating | Phase::Optimizing => {
            plural(status.procedures, "procedure", "procedures")
        }
    };
    let line = format!(
        "{} {detail} ({:.1}s)",
        columns(status.phase.verb()),
        status.elapsed.as_secs_f64()
    );
    truncate(line, width)
}

/// Cut `line` to `width` characters, so it never wraps (a wrapped line cannot be erased).
fn truncate(mut line: String, width: usize) -> String {
    if let Some((cut, _)) = line.char_indices().nth(width) {
        line.truncate(cut);
    }
    line
}

/// A finished line such as `    Finished debug [unoptimized] in 2.31s`, coloured when `color`.
pub fn message_line(verb: &str, text: &str, color: bool) -> String {
    if color {
        format!("\x1b[1;32m{}\x1b[0m {text}", columns(verb))
    } else {
        format!("{} {text}", columns(verb))
    }
}

/// The terminal side, behind one lock that only output and the ticker take.
struct Term {
    mode: Mode,
    name: String,
    started: Option<Instant>,
    /// The status line is on screen.
    drawn: bool,
    /// What was drawn last, to skip redrawing the same text.
    last: String,
    /// No redraw before this: something was just written.
    hold_until: Option<Instant>,
    /// `finish`/`abandon` ran: nothing more is drawn.
    done: bool,
}

static TERM: Mutex<Term> = Mutex::new(Term {
    mode: Mode::Off,
    name: String::new(),
    started: None,
    drawn: false,
    last: String::new(),
    hold_until: None,
    done: false,
});

/// How often the line is redrawn, and how long it waits at the start (a fast build never shows
/// it) and after other output.
const TICK: Duration = Duration::from_millis(80);

const FIRST_DRAW: Duration = Duration::from_millis(150);
const HOLD_AFTER_OUTPUT: Duration = Duration::from_millis(400);

fn lock() -> std::sync::MutexGuard<'static, Term> {
    TERM.lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn write_err(text: &str) {
    use std::io::Write;
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(text.as_bytes());
    let _ = err.flush();
}

/// Starts the status for compiling `name` (`shown` is the main file as the user sees it):
/// prints `   Compiling name (shown)` (`verb` is `Compiling` or `Checking`) and, when `Live`,
/// starts redrawing the status line.
pub fn start(mode: Mode, verb: &str, name: &str, shown: &str) {
    if mode == Mode::Off {
        return;
    }
    {
        let mut term = lock();
        term.mode = mode;
        term.name = name.to_string();
        term.started = Some(Instant::now());
        term.done = false;
    }
    write_err(&format!(
        "{}\n",
        message_line(verb, &format!("{name} ({shown})"), mode == Mode::Live)
    ));
    if mode == Mode::Live {
        start_ticker();
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn start_ticker() {
    LIVE.store(true, Ordering::Relaxed);
    // Another jaic started by compile-time code shares this terminal: it must not draw too.
    // Set before the compiler starts any thread that could read the environment.
    #[allow(unsafe_code)]
    // SAFETY: called on the compile thread before the compiler, parse workers or compile-time
    // code start; the only other thread (macOS's main thread) is blocked waiting for this one.
    unsafe {
        std::env::set_var("JAIC_PROGRESS", "off");
    }
    let spawned = std::thread::Builder::new()
        .name("jaic-progress".into())
        .spawn(|| {
            std::thread::sleep(FIRST_DRAW);
            loop {
                if !tick() {
                    return;
                }
                std::thread::sleep(TICK);
            }
        });
    if spawned.is_err() {
        LIVE.store(false, Ordering::Relaxed);
    }
}

#[cfg(target_arch = "wasm32")]
fn start_ticker() {
}

/// One redraw. Returns whether to keep going.
fn tick() -> bool {
    let mut term = lock();
    if term.done {
        return false;
    }
    let now = Instant::now();
    if term.hold_until.is_some_and(|until| now < until) {
        return true;
    }
    let phase = Phase::ALL
        .get(PHASE.load(Ordering::Relaxed) as usize)
        .copied()
        .unwrap_or(Phase::Checking);
    if phase == Phase::Program {
        return true;
    }
    let status = Status {
        phase,
        name: &term.name,
        files: FILES.load(Ordering::Relaxed),
        modules: MODULES.load(Ordering::Relaxed),
        procedures: PROCEDURES.load(Ordering::Relaxed),
        elapsed: term.started.map_or(Duration::ZERO, |s| now - s),
    };
    let width = terminal_width().saturating_sub(1).max(20);
    let line = status_line(&status, width);
    if term.drawn && line == term.last {
        return true;
    }
    // The verb in bold cyan, as cargo shows work in progress.
    let (verb, rest) = line.split_at(12.min(line.len()));
    write_err(&format!("\r\x1b[1;36m{verb}\x1b[0m{rest}\x1b[K"));
    term.drawn = true;
    term.last = line;
    true
}

/// Erase the status line before something else is written to the terminal, and keep it down
/// for a moment. One relaxed load when there is no status line.
#[inline]
pub fn before_output() {
    if live() {
        erase();
    }
}

#[cold]
fn erase() {
    let mut term = lock();
    if term.drawn {
        write_err("\r\x1b[K");
        term.drawn = false;
    }
    term.hold_until = Some(Instant::now() + HOLD_AFTER_OUTPUT);
}

/// Ends the status: erases the line and prints `    Finished summary in 2.31s`.
pub fn finish(summary: &str) {
    let Some(text) = stop() else {
        return;
    };
    write_err(&format!(
        "{}\n",
        message_line("Finished", &format!("{summary} in {text}"), text_color())
    ));
}

/// A line after `Finished`, such as `     Running hello`.
pub fn announce(verb: &str, text: &str) {
    let mode = lock().mode;
    if mode != Mode::Off {
        write_err(&format!(
            "{}\n",
            message_line(verb, text, mode == Mode::Live)
        ));
    }
}

/// Ends the status without a `Finished` line (the compile failed).
pub fn abandon() {
    stop();
}

fn text_color() -> bool {
    lock().mode == Mode::Live
}

/// Stops drawing and erases the line. The time since `start`, formatted, if it was started.
fn stop() -> Option<String> {
    LIVE.store(false, Ordering::Relaxed);
    let mut term = lock();
    if term.mode == Mode::Off || term.done {
        return None;
    }
    term.done = true;
    if term.drawn {
        write_err("\r\x1b[K");
        term.drawn = false;
    }
    let elapsed = term.started.map_or(Duration::ZERO, |s| s.elapsed());
    Some(format!("{:.2}s", elapsed.as_secs_f64()))
}

/// Columns of the terminal on stderr; 80 when unknown.
fn terminal_width() -> usize {
    if let Some(columns) = std::env::var("COLUMNS")
        .ok()
        .and_then(|c| c.parse::<usize>().ok())
        .filter(|&c| c > 0)
    {
        return columns;
    }
    ioctl_width().unwrap_or(80)
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn ioctl_width() -> Option<usize> {
    #[repr(C)]
    #[derive(Default)]
    struct WinSize {
        rows: u16,
        cols: u16,
        x_pixels: u16,
        y_pixels: u16,
    }
    #[cfg(target_os = "macos")]
    const TIOCGWINSZ: std::ffi::c_ulong = 0x4008_7468;
    #[cfg(target_os = "linux")]
    const TIOCGWINSZ: std::ffi::c_ulong = 0x5413;
    #[allow(unsafe_code)]
    unsafe extern "C" {
        fn ioctl(fd: i32, request: std::ffi::c_ulong, ...) -> i32;
    }
    let mut size = WinSize::default();
    #[allow(unsafe_code)]
    // SAFETY: TIOCGWINSZ writes one `struct winsize` (four u16s) through the pointer, which
    // points at a live, correctly laid out value.
    let result = unsafe { ioctl(2, TIOCGWINSZ, &raw mut size) };
    (result == 0 && size.cols > 0).then_some(size.cols as usize)
}

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
fn ioctl_width() -> Option<usize> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(phase: Phase) -> Status<'static> {
        Status {
            phase,
            name: "hello",
            files: 412,
            modules: 38,
            procedures: 5120,
            elapsed: Duration::from_millis(1440),
        }
    }

    #[test]
    fn lines_name_the_phase_and_counts() {
        let cases = [
            (
                Phase::Checking,
                "    Checking 412 files, 38 modules, 5,120 procedures (1.4s)",
            ),
            (
                Phase::Parsing,
                "     Parsing 412 files, 38 modules, 5,120 procedures (1.4s)",
            ),
            (
                Phase::Running,
                "     Running #run, 412 files, 38 modules, 5,120 procedures (1.4s)",
            ),
            (Phase::Generating, "  Generating 5,120 procedures (1.4s)"),
            (Phase::Optimizing, "  Optimizing 5,120 procedures (1.4s)"),
            (Phase::Linking, "     Linking hello (1.4s)"),
        ];
        for (phase, expected) in cases {
            assert_eq!(status_line(&status(phase), 200), expected);
        }
    }

    #[test]
    fn counts_start_small() {
        let s = Status {
            files: 1,
            modules: 0,
            procedures: 0,
            elapsed: Duration::ZERO,
            ..status(Phase::Checking)
        };
        assert_eq!(status_line(&s, 200), "    Checking 1 file (0.0s)");
    }

    #[test]
    fn lines_never_wrap() {
        let line = status_line(&status(Phase::Running), 30);
        assert_eq!(line.chars().count(), 30);
        assert_eq!(line, "     Running #run, 412 files, ");
    }

    #[test]
    fn numbers_are_grouped() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(999), "999");
        assert_eq!(grouped(1000), "1,000");
        assert_eq!(grouped(1_234_567), "1,234,567");
    }

    #[test]
    fn message_lines_align_like_cargo() {
        assert_eq!(
            message_line("Compiling", "hello (hello.jai)", false),
            "   Compiling hello (hello.jai)"
        );
        assert_eq!(
            message_line("Finished", "check in 0.31s", true),
            "\x1b[1;32m    Finished\x1b[0m check in 0.31s"
        );
    }

    #[test]
    fn mode_follows_terminal_and_environment() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |key: &str| {
                pairs
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, v)| (*v).to_string())
            }
        };
        let term: &[(&str, &str)] = &[("TERM", "xterm-256color")];
        assert_eq!(mode_with(true, false, false, env(term)), Mode::Live);
        assert_eq!(mode_with(false, false, false, env(term)), Mode::Off);
        assert_eq!(mode_with(true, true, false, env(term)), Mode::Off);
        assert_eq!(mode_with(true, false, true, env(term)), Mode::Plain);
        let no_color: &[(&str, &str)] = &[("TERM", "xterm"), ("NO_COLOR", "1")];
        assert_eq!(mode_with(true, false, false, env(no_color)), Mode::Plain);
        let ci: &[(&str, &str)] = &[("TERM", "xterm"), ("CI", "true")];
        assert_eq!(mode_with(true, false, false, env(ci)), Mode::Plain);
        let dumb: &[(&str, &str)] = &[("TERM", "dumb")];
        assert_eq!(mode_with(true, false, false, env(dumb)), Mode::Plain);
        let forced: &[(&str, &str)] = &[("JAIC_PROGRESS", "plain")];
        assert_eq!(mode_with(false, false, false, env(forced)), Mode::Plain);
        let live: &[(&str, &str)] = &[("JAIC_PROGRESS", "live")];
        assert_eq!(mode_with(false, false, false, env(live)), Mode::Live);
        assert_eq!(mode_with(false, true, false, env(live)), Mode::Off);
        let off: &[(&str, &str)] = &[("TERM", "xterm"), ("JAIC_PROGRESS", "off")];
        assert_eq!(mode_with(true, false, false, env(off)), Mode::Off);
    }
}
