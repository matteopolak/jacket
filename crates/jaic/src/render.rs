//! Diagnostics as text: one renderer for `jaic`'s compile and runtime errors, `jailint`'s
//! lints and the command-line tools' own errors.
//!
//! Three layouts (see `docs/compiler/diagnostics.md`):
//!
//! - `Plain`, used when stderr is not a terminal (pipes, CI logs, tests): each tool's
//!   long-standing ASCII text. `jaic` writes `path:line:col: error: message` with the line
//!   below it (`Report::compact`); `jailint` writes the rustc-like block with a `-->` line.
//! - `Ascii`: on a terminal without UTF-8, a rustc-like block with line numbers, context
//!   lines, labels and fix previews.
//! - `Unicode`: the same with box-drawing characters.
//!
//! Colour is independent of the layout. The process-wide choice (`set_style`) is made once
//! by each command-line tool; libraries and the language server leave it at `Style::PLAIN`.
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layout {
    Plain,
    Ascii,
    Unicode,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Style {
    pub layout: Layout,
    pub color: bool,
}

impl Style {
    pub const PLAIN: Style = Style {
        layout: Layout::Plain,
        color: false,
    };
}

/// Bits 0-1: layout; bit 2: colour.
static STYLE: AtomicU8 = AtomicU8::new(0);

/// Set the style every later `render` (and `Diagnostic::render`) uses.
pub fn set_style(style: Style) {
    let layout = match style.layout {
        Layout::Plain => 0,
        Layout::Ascii => 1,
        Layout::Unicode => 2,
    };
    STYLE.store(layout | (u8::from(style.color) << 2), Ordering::Relaxed);
}

pub fn style() -> Style {
    let bits = STYLE.load(Ordering::Relaxed);
    Style {
        layout: match bits & 3 {
            1 => Layout::Ascii,
            2 => Layout::Unicode,
            _ => Layout::Plain,
        },
        color: bits & 4 != 0,
    }
}

/// A `--color` value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColorChoice {
    Auto,
    Always,
    Never,
}

impl ColorChoice {
    pub fn parse(value: &str) -> Option<ColorChoice> {
        match value {
            "auto" => Some(ColorChoice::Auto),
            "always" => Some(ColorChoice::Always),
            "never" => Some(ColorChoice::Never),
            _ => None,
        }
    }
}

/// The style for diagnostics on stderr, from `choice` (`--color`) and the environment.
pub fn detect(choice: ColorChoice) -> Style {
    use std::io::IsTerminal;
    detect_with(choice, std::io::stderr().is_terminal(), |key| {
        std::env::var(key).ok()
    })
}

/// `detect` with the terminal test and the environment given (for tests).
///
/// - Colour: `--color always|never` wins; otherwise `NO_COLOR` (any value) turns it off,
///   `CLICOLOR_FORCE` / `FORCE_COLOR` (set, not `0`) turn it on, and else it is on for a
///   terminal whose `TERM` is not `dumb`. On Windows, `auto` colours only consoles known to
///   understand ANSI sequences (Windows Terminal, ConEmu, ANSICON, a `TERM`).
/// - Layout: `JAIC_DIAGNOSTICS=plain|ascii|unicode` wins; otherwise `Plain` when stderr is
///   not a terminal (or `TERM=dumb`), `Unicode` for a UTF-8 locale or Windows Terminal, and
///   `Ascii` for other terminals.
pub fn detect_with(
    choice: ColorChoice,
    terminal: bool,
    env: impl Fn(&str) -> Option<String>,
) -> Style {
    let forced = |key: &str| env(key).is_some_and(|v| !v.is_empty() && v != "0");
    let dumb = env("TERM").as_deref() == Some("dumb");
    let ansi_console = !cfg!(windows)
        || env("WT_SESSION").is_some()
        || env("ANSICON").is_some()
        || env("ConEmuANSI").as_deref() == Some("ON")
        || env("TERM").is_some();
    let color = match choice {
        ColorChoice::Always => true,
        ColorChoice::Never => false,
        ColorChoice::Auto if env("NO_COLOR").is_some() => false,
        ColorChoice::Auto if forced("CLICOLOR_FORCE") || forced("FORCE_COLOR") => true,
        ColorChoice::Auto => terminal && !dumb && ansi_console,
    };
    let utf8 = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .find_map(|key| env(key).filter(|v| !v.is_empty()))
        .is_some_and(|v| {
            let v = v.to_ascii_lowercase();
            v.contains("utf-8") || v.contains("utf8")
        });
    let layout = match env("JAIC_DIAGNOSTICS").as_deref() {
        Some("plain") => Layout::Plain,
        Some("ascii") => Layout::Ascii,
        Some("unicode") => Layout::Unicode,
        _ if !terminal || dumb => Layout::Plain,
        _ if utf8 || env("WT_SESSION").is_some() => Layout::Unicode,
        _ => Layout::Ascii,
    };
    Style {
        layout,
        color,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }

    fn color(self) -> &'static str {
        match self {
            Severity::Error => "1;31",
            Severity::Warning => "1;33",
            Severity::Note => "1;36",
            Severity::Help => "1;32",
        }
    }
}

/// A place in a source file, with the file's text.
#[derive(Clone, Debug)]
pub struct Label<'a> {
    pub path: &'a str,
    pub text: &'a str,
    /// Byte range in `text`.
    pub start: usize,
    pub end: usize,
    pub message: Option<String>,
}

/// A fix to show under its help line: the replaced ranges of `text` and their new text.
#[derive(Clone, Debug)]
pub struct Fix<'a> {
    pub text: &'a str,
    /// (start, end, replacement), byte ranges of `text`.
    pub edits: Vec<(usize, usize, String)>,
}

#[derive(Clone, Debug)]
pub struct Help<'a> {
    pub message: String,
    pub fix: Option<Fix<'a>>,
}

#[derive(Clone, Debug)]
pub struct Report<'a> {
    pub severity: Severity,
    /// `[name]` after the severity (a lint's rule).
    pub code: Option<&'a str>,
    pub message: String,
    /// Where it happened; `None` for problems without a place in a file.
    pub primary: Option<Label<'a>>,
    /// Related places. In the plain compact layout each prints as its own `note:`.
    pub secondary: Vec<Label<'a>>,
    /// `note:` text without a place.
    pub notes: Vec<String>,
    pub help: Vec<Help<'a>>,
    /// Plain layout: `jaic`'s `path:line:col: error: message` form (rather than `-->`).
    pub compact: bool,
}

impl<'a> Report<'a> {
    pub fn new(severity: Severity, message: impl Into<String>) -> Self {
        Report {
            severity,
            code: None,
            message: message.into(),
            primary: None,
            secondary: Vec::new(),
            notes: Vec::new(),
            help: Vec::new(),
            compact: true,
        }
    }

    pub fn help(mut self, message: impl Into<String>) -> Self {
        self.help.push(Help {
            message: message.into(),
            fix: None,
        });
        self
    }

    pub fn note(mut self, message: impl Into<String>) -> Self {
        self.notes.push(message.into());
        self
    }

    /// The report in the current process-wide style.
    pub fn render(&self) -> String {
        self.render_with(style())
    }

    pub fn render_with(&self, style: Style) -> String {
        let mut r = Renderer {
            paint: Paint(style.color),
            glyphs: if style.layout == Layout::Unicode {
                &UNICODE
            } else {
                &ASCII
            },
            plain: style.layout == Layout::Plain,
            severity_color: self.severity.color(),
            out: String::new(),
        };
        if r.plain && self.compact {
            r.compact(self);
        } else {
            r.block(self);
        }
        r.out
    }
}

/// Longest stretch of changed code shown under a help.
const MAX_FIX_LINES: usize = 8;
/// Lines shown around each label outside the plain layout.
const CONTEXT_LINES: usize = 1;
/// Lines of a long multi-line label shown at its start and end.
const SPAN_EDGE_LINES: usize = 2;
/// Source lines longer than this (in characters) are cut down around their labels.
const MAX_LINE_WIDTH: usize = 120;
/// Spaces a tab stands for outside the plain layout.
const TAB_WIDTH: usize = 4;

struct Glyphs {
    /// Top of a snippet, before `path:line:col`.
    open: &'static str,
    close: &'static str,
    /// The gutter beside source lines, and beside annotations.
    bar: &'static str,
    note_bar: &'static str,
    gap: &'static str,
    primary: char,
    secondary: char,
    /// A multi-line label's connector: first line, lines after, the closing annotation.
    span_start: &'static str,
    span_mid: &'static str,
    span_end: &'static str,
    span_line: char,
    ellipsis: &'static str,
}

const ASCII: Glyphs = Glyphs {
    open: "-->",
    close: "",
    bar: "|",
    note_bar: "|",
    gap: "...",
    primary: '^',
    secondary: '-',
    span_start: "/",
    span_mid: "|",
    span_end: "\\",
    span_line: '_',
    ellipsis: "...",
};

const UNICODE: Glyphs = Glyphs {
    open: "╭─[",
    close: "╰─",
    bar: "│",
    note_bar: "·",
    gap: "┆",
    primary: '━',
    secondary: '─',
    span_start: "╭",
    span_mid: "│",
    span_end: "╰",
    span_line: '─',
    ellipsis: "…",
};

#[derive(Clone, Copy)]
struct Paint(bool);

impl Paint {
    fn on(&self, code: &str, text: &str) -> String {
        if self.0 && !text.is_empty() {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }
}

const GUTTER: &str = "34";
const SECONDARY: &str = "1;34";

struct Renderer {
    paint: Paint,
    glyphs: &'static Glyphs,
    plain: bool,
    /// The colour of the report's severity, for its primary label.
    severity_color: &'static str,
    out: String,
}

/// One-based line and column (in characters) of a byte offset.
pub fn line_col(text: &str, offset: usize) -> (usize, usize) {
    let offset = text.floor_char_boundary(offset.min(text.len()));
    let line_start = text[..offset].rfind('\n').map_or(0, |n| n + 1);
    let line = text[..offset].matches('\n').count() + 1;
    (line, text[line_start..offset].chars().count() + 1)
}

/// Byte range of one-based `line` (without its newline).
fn line_range(text: &str, line: usize) -> Option<(usize, usize)> {
    let mut start = 0;
    for _ in 1..line {
        start += text[start..].find('\n')? + 1;
    }
    let end = text[start..].find('\n').map_or(text.len(), |n| start + n);
    let end = if text[start..end].ends_with('\r') {
        end - 1
    } else {
        end
    };
    Some((start, end))
}

fn line_count(text: &str) -> usize {
    text.matches('\n').count() + usize::from(!text.ends_with('\n') || text.is_empty())
}

impl Renderer {
    fn line(&mut self, text: &str) {
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn header(&mut self, severity: Severity, code: Option<&str>, message: &str) -> String {
        let head = match code {
            Some(code) => format!("{}[{code}]", severity.as_str()),
            None => severity.as_str().to_string(),
        };
        format!(
            "{}{}",
            self.paint.on(severity.color(), &head),
            self.paint.on("1", &format!(": {message}"))
        )
    }

    /// `jaic`'s plain form: each located item as `path:line:col: kind: message`, its line
    /// indented by four, and carets under the range on that line.
    fn compact(&mut self, report: &Report) {
        let items = std::iter::once((report.severity, &report.message, report.primary.as_ref()))
            .chain(report.secondary.iter().map(|l| {
                (
                    Severity::Note,
                    l.message.as_ref().unwrap_or(&report.message),
                    Some(l),
                )
            }));
        let items: Vec<_> = items.collect();
        for (severity, message, label) in items {
            let Some(label) = label else {
                let head = self.header(severity, None, message);
                self.line(&head);
                continue;
            };
            let (line, col) = line_col(label.text, label.start);
            let head = self.header(severity, None, message);
            let location = format!("{}:{line}:{col}: ", label.path);
            self.line(&format!("{location}{head}"));
            let (start, end) = line_range(label.text, line).unwrap_or((0, 0));
            let source = &label.text[start..end];
            self.line(&format!("    {source}"));
            let pad: String = source
                .chars()
                .take(col - 1)
                .map(|c| {
                    if c == '\t' {
                        '\t'
                    } else {
                        ' '
                    }
                })
                .collect();
            let first = label.start.clamp(start, end);
            let width = label.text[first..label.end.clamp(first, end)]
                .chars()
                .count();
            let width = width.clamp(1, (source.chars().count() + 1).saturating_sub(col).max(1));
            let carets = self.paint.on(severity.color(), &"^".repeat(width));
            self.line(&format!("    {pad}{carets}"));
        }
        for note in &report.notes {
            let head = self.header(Severity::Note, None, note);
            self.line(&head);
        }
        for help in &report.help {
            let head = self.header(Severity::Help, None, &help.message);
            self.line(&head);
        }
    }

    /// The block form: header, `-->` line, numbered source lines with labels, then notes and
    /// help.
    fn block(&mut self, report: &Report) {
        let head = self.header(report.severity, report.code, &report.message);
        self.line(&head);
        let mut labels: Vec<(&Label, bool)> = report.primary.iter().map(|l| (l, true)).collect();
        labels.extend(report.secondary.iter().map(|l| (l, false)));
        let width = self.gutter_width(&labels);
        // One section per file, the primary label's first.
        let mut files: Vec<&str> = Vec::new();
        for (label, _) in &labels {
            if !files.contains(&label.path) {
                files.push(label.path);
            }
        }
        let g = self.glyphs;
        let blank = " ".repeat(width);
        let bar = self.paint.on(GUTTER, g.bar);
        for (i, path) in files.iter().enumerate() {
            let in_file: Vec<(&Label, bool)> = labels
                .iter()
                .copied()
                .filter(|(l, _)| l.path == *path)
                .collect();
            let (first, _) = in_file[0];
            let (line, col) = line_col(first.text, first.start);
            if self.plain || g.close.is_empty() {
                let arrow = if i == 0 {
                    g.open
                } else {
                    ":::"
                };
                self.line(&format!(
                    "{blank}{} {path}:{line}:{col}",
                    self.paint.on(GUTTER, arrow)
                ));
                self.line(&format!("{blank} {bar}"));
            } else {
                let open = self.paint.on(GUTTER, g.open);
                self.line(&format!(
                    "{blank} {open}{path}:{line}:{col}{}",
                    self.paint.on(GUTTER, "]")
                ));
            }
            self.snippet(&in_file, width);
            if self.plain || g.close.is_empty() {
                self.line(&format!("{blank} {bar}"));
            } else {
                self.line(&format!("{blank} {}", self.paint.on(GUTTER, g.close)));
            }
        }
        let note_head = |r: &Renderer, severity: Severity, text: &str| {
            let indent = if labels.is_empty() {
                String::new()
            } else {
                format!("{blank} {} ", r.paint.on(GUTTER, "="))
            };
            let continuation = " ".repeat(indent_width(&indent, r.paint.0));
            let mut lines = text.lines();
            let mut out = format!(
                "{indent}{}: {}",
                r.paint.on("1", severity.as_str()),
                lines.next().unwrap_or("")
            );
            for rest in lines {
                out.push('\n');
                out.push_str(&continuation);
                out.push_str(rest);
            }
            out
        };
        for help in &report.help {
            match &help.fix {
                Some(fix) if !fix.edits.is_empty() && !labels.is_empty() => {
                    self.line(&format!(
                        "{}: {}",
                        self.paint.on(Severity::Help.color(), "help"),
                        help.message
                    ));
                    self.line(&format!("{blank} {bar}"));
                    for (n, mark, code) in changed_lines(fix.text, &fix.edits) {
                        let paint = match mark {
                            '-' => "31",
                            '+' => "32",
                            _ => GUTTER,
                        };
                        let code = if self.plain {
                            code
                        } else {
                            expand_tabs(&code)
                        };
                        self.line(&format!(
                            "{} {} {code}",
                            self.paint.on(GUTTER, &format!("{n:>width$}")),
                            self.paint.on(paint, &mark.to_string())
                        ));
                    }
                    self.line(&format!("{blank} {bar}"));
                }
                _ => {
                    let text = note_head(self, Severity::Help, &help.message);
                    self.line(&text);
                }
            }
        }
        for note in &report.notes {
            let text = note_head(self, Severity::Note, note);
            self.line(&text);
        }
    }

    fn gutter_width(&self, labels: &[(&Label, bool)]) -> usize {
        let last = labels
            .iter()
            .map(|(l, _)| {
                let (line, _) = line_col(l.text, l.start);
                let (end, _) = line_col(l.text, l.end.max(l.start));
                end.max(line + MAX_FIX_LINES) + CONTEXT_LINES
            })
            .max()
            .unwrap_or(1);
        if self.plain {
            // As jailint has always laid it out.
            labels
                .iter()
                .map(|(l, _)| {
                    let (line, _) = line_col(l.text, l.start);
                    let (end, _) = line_col(l.text, l.end.max(l.start));
                    end.max(line + 8)
                })
                .max()
                .unwrap_or(1)
                .to_string()
                .len()
        } else {
            last.to_string().len()
        }
    }

    /// The numbered lines of one file with its labels.
    fn snippet(&mut self, labels: &[(&Label, bool)], width: usize) {
        let g = self.glyphs;
        let text = labels[0].0.text;
        let total = line_count(text);
        let context = if self.plain {
            0
        } else {
            CONTEXT_LINES
        };
        // The one label drawn across lines (outside the plain layout, which shows a
        // label's first line only).
        let spans: Vec<(usize, usize, usize, usize)> = labels
            .iter()
            .map(|(l, _)| {
                let (a, ac) = line_col(text, l.start);
                let (b, bc) = line_col(text, l.end.max(l.start + 1).saturating_sub(1).max(l.start));
                (a, ac, b, bc)
            })
            .collect();
        let multi = if self.plain {
            None
        } else {
            spans.iter().position(|s| s.2 > s.0)
        };
        // The lines to show: around each label; a long multi-line label shows its edges.
        let mut shown: Vec<usize> = Vec::new();
        for (i, s) in spans.iter().enumerate() {
            let end = if Some(i) == multi {
                s.2
            } else {
                s.0
            };
            let mut add = |from: usize, to: usize| {
                for n in from.max(1)..=to.min(total) {
                    shown.push(n);
                }
            };
            if end - s.0 > 2 * SPAN_EDGE_LINES + 1 {
                add(s.0.saturating_sub(context), s.0 + SPAN_EDGE_LINES - 1);
                add(end + 1 - SPAN_EDGE_LINES, end + context);
            } else {
                add(s.0.saturating_sub(context), end + context);
            }
        }
        shown.sort_unstable();
        shown.dedup();
        let bar = self.paint.on(GUTTER, g.bar);
        let note_bar = self.paint.on(GUTTER, g.note_bar);
        let blank = " ".repeat(width);
        let connector_width = if multi.is_some() {
            2
        } else {
            0
        };
        let mut previous = 0;
        for &n in &shown {
            if previous != 0 && n > previous + 1 {
                self.line(&format!(
                    "{} {}",
                    " ".repeat(width),
                    self.paint.on(GUTTER, g.gap)
                ));
            }
            previous = n;
            let (start, end) = line_range(text, n).unwrap_or((text.len(), text.len()));
            let source = &text[start..end];
            // Labels on this line: (first column, last column + 1, label, primary), columns
            // in characters of the shown text.
            let mut marks: Vec<(usize, usize, &Label, bool)> = Vec::new();
            for (i, (label, primary)) in labels.iter().enumerate() {
                let s = spans[i];
                if Some(i) == multi || s.0 != n {
                    continue;
                }
                let from = label.start.clamp(start, end);
                let to = label.end.clamp(from, end);
                let a = display_width(&text[start..from], self.plain);
                let b = a + display_width(&text[from..to], self.plain).max(1);
                marks.push((a, b, label, *primary));
            }
            let (shown_text, offset) = if self.plain {
                (source.trim_end().to_string(), 0)
            } else {
                trim_line(&expand_tabs(source), &marks, g.ellipsis)
            };
            let connector = match multi.map(|m| spans[m]) {
                Some(s) if n == s.0 => format!("{} ", self.paint.on(SECONDARY, g.span_start)),
                Some(s) if n > s.0 && n <= s.2 => {
                    format!("{} ", self.paint.on(SECONDARY, g.span_mid))
                }
                Some(_) => "  ".to_string(),
                None => String::new(),
            };
            let number = self.paint.on(GUTTER, &format!("{n:>width$}"));
            let line = format!("{number} {bar} {connector}{shown_text}");
            self.line(line.trim_end());
            // Primary label first, then the others left to right.
            marks.sort_by_key(|m| (!m.3, m.0));
            for (a, b, label, primary) in marks {
                let (a, b) = (a.saturating_sub(offset), b.saturating_sub(offset));
                let pad: String = if self.plain {
                    source
                        .chars()
                        .take(a)
                        .map(|c| {
                            if c == '\t' {
                                '\t'
                            } else {
                                ' '
                            }
                        })
                        .collect()
                } else {
                    " ".repeat(a)
                };
                let (mark, color) = if primary {
                    (
                        if self.plain {
                            '^'
                        } else {
                            g.primary
                        },
                        self.severity_color,
                    )
                } else {
                    (
                        if self.plain {
                            '-'
                        } else {
                            g.secondary
                        },
                        SECONDARY,
                    )
                };
                let mut underline = self.paint.on(color, &mark.to_string().repeat(b - a));
                if let Some(message) = &label.message {
                    underline.push(' ');
                    underline.push_str(&self.paint.on(color, message));
                }
                let side = if self.plain {
                    &bar
                } else {
                    &note_bar
                };
                let lead = if connector_width > 0 {
                    match multi.map(|m| spans[m]) {
                        Some(s) if n >= s.0 && n < s.2 => {
                            format!("{} ", self.paint.on(SECONDARY, g.span_mid))
                        }
                        _ => "  ".to_string(),
                    }
                } else {
                    String::new()
                };
                self.line(&format!("{blank} {side} {lead}{pad}{underline}"));
            }
            // After a multi-line label's last line: close it, under its end.
            if let Some(m) = multi
                && spans[m].2 == n
            {
                let (label, primary) = labels[m];
                let last =
                    display_width(&text[start..label.end.clamp(start, end).max(start)], false)
                        .max(1);
                let color = if primary {
                    self.severity_color
                } else {
                    SECONDARY
                };
                let mut close =
                    format!("{}{}", g.span_end, g.span_line.to_string().repeat(last + 1));
                if let Some(message) = &label.message {
                    close.push(' ');
                    close.push_str(message);
                }
                self.line(&format!(
                    "{blank} {note_bar} {}",
                    self.paint.on(color, &close)
                ));
            }
        }
    }
}

/// Characters `text` takes on screen (tabs as `TAB_WIDTH` spaces unless `plain`).
fn display_width(text: &str, plain: bool) -> usize {
    if plain {
        text.chars().count()
    } else {
        text.chars()
            .map(|c| {
                if c == '\t' {
                    TAB_WIDTH
                } else {
                    1
                }
            })
            .sum()
    }
}

fn expand_tabs(text: &str) -> String {
    text.replace('\t', &" ".repeat(TAB_WIDTH))
}

/// A line longer than `MAX_LINE_WIDTH` cut down to the stretch around its marks; returns the
/// text and how many characters were cut from the front (the marks' columns move left).
fn trim_line(
    line: &str,
    marks: &[(usize, usize, &Label, bool)],
    ellipsis: &str,
) -> (String, usize) {
    let line = line.trim_end();
    let chars: Vec<char> = line.chars().collect();
    if chars.len() <= MAX_LINE_WIDTH {
        return (line.to_string(), 0);
    }
    let first = marks.iter().map(|m| m.0).min().unwrap_or(0);
    let last = marks.iter().map(|m| m.1).max().unwrap_or(0);
    let margin = MAX_LINE_WIDTH.saturating_sub(last.saturating_sub(first)) / 2;
    let from = first.saturating_sub(margin.max(8));
    let to = (last + margin.max(8)).min(chars.len());
    let mut out = String::new();
    let mut offset = from;
    if from > 0 {
        out.push_str(ellipsis);
        offset = from.saturating_sub(ellipsis.chars().count());
    }
    out.extend(&chars[from..to]);
    if to < chars.len() {
        out.push_str(ellipsis);
    }
    (out, offset)
}

/// Visible width of `text` (ANSI colour sequences take none).
fn indent_width(text: &str, colored: bool) -> usize {
    if !colored {
        return text.chars().count();
    }
    let mut width = 0;
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            width += 1;
        }
    }
    width
}

/// The code a fix changes, as (line number, mark, text). When the fix keeps the number of
/// lines, the changed lines as fixed (`~`); when it removes whole lines, those (`-`);
/// otherwise the old lines (`-`) then the new (`+`). At most `MAX_FIX_LINES` of each.
pub fn changed_lines(text: &str, edits: &[(usize, usize, String)]) -> Vec<(String, char, String)> {
    let mut edits = edits.to_vec();
    edits.sort_by_key(|e| e.0);
    let first = edits.first().map_or(0, |e| e.0);
    let last = edits.iter().map(|e| e.1).max().unwrap_or(0);
    let mut fixed = text.to_string();
    for (start, end, new) in edits.iter().rev() {
        fixed.replace_range(*start..*end, new);
    }
    // Where the last edit ends in the fixed text.
    let mut shift: isize = 0;
    let mut last_new = 0usize;
    for (start, end, new) in &edits {
        let at = (*start as isize + shift) as usize;
        last_new = last_new.max(at + new.len());
        shift += new.len() as isize - (end - start) as isize;
    }
    let (first_line, _) = line_col(text, first);
    // An edit ending at a line's start (a removed whole line) ends on the line before.
    let (old_last, _) = line_col(text, last.saturating_sub(1).max(first));
    let (new_last, _) = line_col(&fixed, last_new.saturating_sub(1).max(first));
    let old: Vec<&str> = text.lines().collect();
    let new: Vec<&str> = fixed.lines().collect();
    let take = |lines: &[&str], from: usize, to: usize, mark: char, out: &mut Vec<_>| {
        for n in from..=to.max(from) {
            if n - from == MAX_FIX_LINES {
                out.push(("...".to_string(), mark, String::new()));
                break;
            }
            if let Some(l) = lines.get(n - 1) {
                out.push((n.to_string(), mark, l.trim_end().to_string()));
            }
        }
    };
    let mut out = Vec::new();
    let removed = |e: &(usize, usize, String)| text[e.0..e.1].matches('\n').count();
    let added = |e: &(usize, usize, String)| e.2.matches('\n').count();
    if edits.iter().all(|e| removed(e) == added(e)) {
        take(&new, first_line, new_last, '~', &mut out);
    } else if edits.iter().all(|e| {
        let line_start = e.0 == 0 || text[..e.0].ends_with('\n');
        e.2.is_empty() && e.1 > e.0 && line_start && text[..e.1].ends_with('\n')
    }) {
        for e in &edits {
            let (a, _) = line_col(text, e.0);
            let (b, _) = line_col(text, e.1 - 1);
            take(&old, a, b, '-', &mut out);
        }
    } else {
        take(&old, first_line, old_last, '-', &mut out);
        take(&new, first_line, new_last, '+', &mut out);
    }
    out
}

#[cfg(test)]
mod tests;
