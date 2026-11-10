//! The notes of a `// test:` comment at the top of a source file, and the target filters
//! (`only(.WINDOWS)`, `skip_on(.WASM)`) among them. `jacket test` reads them to decide what to
//! run; `jailint` reads them to decide what to compile a file as.

use std::fmt;
use std::path::Path;

/// The folder below `tests/` whose `.jai` files must fail to compile, notes or not.
pub const COMPILE_FAIL_DIR: &str = "compile_fail";

/// Whether `path` is a `tests/compile_fail/*.jai` fixture, which `jacket test` expects to fail to
/// compile without a `compile_fail` note.
///
/// ```
/// use jaic::test_notes::in_compile_fail_folder;
/// use std::path::Path;
/// assert!(in_compile_fail_folder(Path::new("Csv/tests/compile_fail/bad.jai")));
/// assert!(!in_compile_fail_folder(Path::new("Csv/tests/bad.jai")));
/// ```
pub fn in_compile_fail_folder(path: &Path) -> bool {
    let folder = path.parent();
    folder.and_then(Path::file_name) == Some(COMPILE_FAIL_DIR.as_ref())
        && folder.and_then(Path::parent).and_then(Path::file_name) == Some("tests".as_ref())
}


/// Where tests run: the target and how.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub os: String,
    pub cpu: String,
    pub interp: bool,
}

pub const OS_NAMES: [&str; 5] = ["linux", "macos", "windows", "wasm", "android"];
pub const CPU_NAMES: [&str; 2] = ["x64", "arm64"];
pub const MODE_NAMES: [&str; 2] = ["interp", "native"];

/// The arguments of one `only(...)` or `skip_on(...)` call: target names (an OS, a CPU or a
/// mode, as in `.WASM`, `.ARM64`, `.INTERP`), and `mode = .NATIVE` or `cpu = .ARM64`, which
/// narrow the whole call, and `reason = "text"`, which only explains it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetFilter {
    pub names: Vec<String>,
    pub mode: Option<String>,
    pub cpu: Option<String>,
    /// Why the call is there (`reason = "needs threads"`), shown with the skip.
    pub reason: Option<String>,
}

impl TargetFilter {
    /// `.WASM, .WINDOWS, cpu = .ARM64` as a filter. `call` is `only` or `skip_on`.
    pub fn parse(call: &str, args: &str) -> Result<TargetFilter, String> {
        let mut filter = TargetFilter::default();
        for item in split_outside_quotes(args)
            .iter()
            .map(|i| i.trim())
            .filter(|i| !i.is_empty())
        {
            if let Some((key, value)) = item.split_once('=') {
                let key = key.trim();
                if key == "reason" {
                    let reason = text(Some(value));
                    if reason.is_empty() || filter.reason.replace(reason).is_some() {
                        return Err(format!("`@{call}` takes one non-empty `reason = \"text\"`"));
                    }
                    continue;
                }
                let value = value.trim().trim_start_matches('.').to_ascii_lowercase();
                let (slot, allowed) = match key {
                    "mode" => (&mut filter.mode, &MODE_NAMES),
                    "cpu" => (&mut filter.cpu, &CPU_NAMES),
                    _ => {
                        return Err(format!(
                            "`@{call}` does not know the argument `{key}`; use `mode =`, `cpu =` or `reason =`"
                        ));
                    }
                };
                if !allowed.contains(&value.as_str()) {
                    return Err(format!(
                        "`@{call}` does not know the {key} `{value}`; use {}",
                        dotted(allowed)
                    ));
                }
                if slot.replace(value).is_some() {
                    return Err(format!("`@{call}` gives `{key} =` twice"));
                }
                continue;
            }
            let name = item.trim_start_matches('.').to_ascii_lowercase();
            if ![&OS_NAMES[..], &CPU_NAMES[..], &MODE_NAMES[..]]
                .iter()
                .any(|names| names.contains(&name.as_str()))
            {
                return Err(format!(
                    "`@{call}` does not know the target `{name}`; use .LINUX, .MACOS, .WINDOWS, .WASM, .ANDROID, .X64, .ARM64, .INTERP or .NATIVE, and `mode =` or `cpu =` to narrow them"
                ));
            }
            filter.names.push(name);
        }
        if filter.names.is_empty() && filter.mode.is_none() && filter.cpu.is_none() {
            return Err(format!("`@{call}` needs targets, as in `@{call}(.LINUX)`"));
        }
        Ok(filter)
    }

    pub fn reason_suffix(&self) -> String {
        self.reason
            .as_ref()
            .map_or_else(String::new, |r| format!(": {r}"))
    }

    pub fn named(target: &Target, mode: &str, name: &str) -> bool {
        name == target.os || name == target.cpu || name == mode
    }

    /// `mode =` and `cpu =`, which every target of the call must satisfy.
    fn narrowed(&self, target: &Target, mode: &str) -> bool {
        self.mode.as_deref().is_none_or(|m| m == mode)
            && self.cpu.as_deref().is_none_or(|c| c == target.cpu)
    }

    /// Whether `target` is one the call names: any of the names (any OS if there are none), and
    /// the mode and CPU.
    pub fn matches(&self, target: &Target, mode: &str) -> bool {
        (self.names.is_empty() || self.names.iter().any(|n| Self::named(target, mode, n)))
            && self.narrowed(target, mode)
    }

    /// Whether `target` passes an `only` call: each kind of name given (OS, CPU, mode) must
    /// match one of that kind, so `only(.LINUX, .X64)` is Linux on x64.
    pub fn allows(&self, target: &Target, mode: &str) -> bool {
        [&OS_NAMES[..], &CPU_NAMES[..], &MODE_NAMES[..]]
            .iter()
            .all(|kind| {
                let mut named = self
                    .names
                    .iter()
                    .filter(|n| kind.contains(&n.as_str()))
                    .peekable();
                named.peek().is_none() || named.any(|n| Self::named(target, mode, n))
            })
            && self.narrowed(target, mode)
    }
}

/// What a file's notes say about where it runs: `only(...)`, `skip_on(...)`, `interp_only` and
/// `native_only`. `jacket test` decides per test with it, `jailint` per file.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetNotes {
    /// One filter per `only(...)` call; a target must pass every one.
    pub only: Vec<TargetFilter>,
    /// One filter per `skip_on(...)` call; a target matching any is skipped.
    pub skip_on: Vec<TargetFilter>,
    pub interp_only: bool,
    pub native_only: bool,
}

impl TargetNotes {
    /// Takes in one note (`only(.LINUX)`, `native_only`). `Ok(false)` if it is not a note about
    /// targets; `Err` if it is one and does not parse (the note is then ignored).
    ///
    /// ```
    /// # use jaic::test_notes::TargetNotes;
    /// let mut notes = TargetNotes::default();
    /// assert_eq!(notes.apply("skip_on(.WASM)"), Ok(true));
    /// assert_eq!(notes.apply("timeout(5)"), Ok(false));
    /// assert!(notes.apply("only(.NOWHERE)").is_err());
    /// ```
    pub fn apply(&mut self, note: &str) -> Result<bool, String> {
        let (call, args) = match note.split_once('(') {
            Some((call, rest)) => match rest.strip_suffix(')') {
                Some(args) => (call, args),
                None if matches!(call, "only" | "skip_on") => {
                    return Err(format!("`@{note}` has no closing `)`"));
                }
                None => (call, rest),
            },
            None => (note, ""),
        };
        match call {
            "only" => self.only.push(TargetFilter::parse(call, args)?),
            "skip_on" => self.skip_on.push(TargetFilter::parse(call, args)?),
            "interp_only" => self.interp_only = true,
            "native_only" => self.native_only = true,
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// Whether any note restricts where the file runs.
    pub fn restricts(&self) -> bool {
        !self.only.is_empty() || !self.skip_on.is_empty() || self.interp_only || self.native_only
    }

    /// Why a file with these notes does not run on `target`, if it does not.
    pub fn skip_reason(&self, target: &Target) -> Option<String> {
        if self.interp_only && !target.interp {
            return Some("interp only".into());
        }
        if self.native_only && target.interp {
            return Some("native only".into());
        }
        let mode = if target.interp {
            "interp"
        } else {
            "native"
        };
        if let Some(filter) = self.only.iter().find(|f| !f.allows(target, mode)) {
            return Some(format!("only on {filter}{}", filter.reason_suffix()));
        }
        if let Some(filter) = self.skip_on.iter().find(|f| f.matches(target, mode)) {
            // Name the targets that matched, not every one the call lists.
            let matched = TargetFilter {
                names: filter
                    .names
                    .iter()
                    .filter(|n| TargetFilter::named(target, mode, n))
                    .cloned()
                    .collect(),
                ..filter.clone()
            };
            return Some(format!("skipped on {matched}{}", filter.reason_suffix()));
        }
        None
    }

    /// Whether a file with these notes runs on `os` and `cpu` in either run mode, for callers
    /// that have no run mode (linting).
    pub fn runs_on(&self, os: &str, cpu: &str) -> bool {
        [true, false].into_iter().any(|interp| {
            let target = Target {
                os: os.into(),
                cpu: cpu.into(),
                interp,
            };
            self.skip_reason(&target).is_none()
        })
    }
}

/// The target notes in the header of `source`, and a problem for each that does not parse.
pub fn header_target_notes(source: &str) -> (TargetNotes, Vec<String>) {
    let mut notes = TargetNotes::default();
    let problems = header_notes(source)
        .iter()
        .filter_map(|note| notes.apply(note).err())
        .collect();
    (notes, problems)
}

impl fmt::Display for TargetFilter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = self.names.clone();
        parts.extend(self.mode.iter().map(|m| format!("mode = {m}")));
        parts.extend(self.cpu.iter().map(|c| format!("cpu = {c}")));
        f.write_str(&parts.join(", "))
    }
}

/// `a, "b, c", d` as `["a", " \"b, c\"", " d"]`: commas inside quotes do not split.
fn split_outside_quotes(args: &str) -> Vec<String> {
    let mut items = vec![String::new()];
    let mut quoted = false;
    let mut chars = args.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' if quoted => {
                items.last_mut().unwrap().push(c);
                items.last_mut().unwrap().extend(chars.next());
                continue;
            }
            '"' => quoted = !quoted,
            ',' if !quoted => {
                items.push(String::new());
                continue;
            }
            _ => {}
        }
        items.last_mut().unwrap().push(c);
    }
    items
}

fn dotted(names: &[&str]) -> String {
    let named: Vec<String> = names.iter().map(|n| n.to_ascii_uppercase()).collect();
    format!(".{}", named.join(" or ."))
}


/// A note's argument as written (case kept), without the quotes around it.
pub fn text(args: Option<&str>) -> String {
    let args = args.unwrap_or("").trim();
    args.strip_prefix('"')
        .and_then(|a| a.strip_suffix('"'))
        .unwrap_or(args)
        .to_string()
}


/// The notes of the `// test:` comments at the top of `source`, before its first line of code.
pub fn header_notes(source: &str) -> Vec<String> {
    let mut notes = Vec::new();
    for line in source.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some(comment) = line.strip_prefix("//") else {
            break;
        };
        if let Some(items) = comment.trim().strip_prefix("test:") {
            notes.extend(split_notes(items));
        }
    }
    notes
}

/// `timeout(10) only(.LINUX, .MACOS)` as `["timeout(10)", "only(.LINUX,.MACOS)"]`: spaces
/// inside parentheses do not split.
fn split_notes(items: &str) -> Vec<String> {
    let mut notes = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut escaped = false;
    for c in items.chars() {
        if std::mem::take(&mut escaped) {
            current.push(c);
            continue;
        }
        match c {
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '(' if !quoted => depth += 1,
            ')' if !quoted => depth = depth.saturating_sub(1),
            _ => {}
        }
        if c.is_whitespace() && depth == 0 && !quoted {
            if !current.is_empty() {
                notes.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(c);
    }
    if !current.is_empty() {
        notes.push(current);
    }
    notes
        .into_iter()
        .map(|n| n.trim_start_matches('@').to_string())
        .collect()
}
