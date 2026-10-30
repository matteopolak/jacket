//! Printing results: a line per test as it finishes, then the failures in full and a summary
//! (`--format human`); or one JSON object per event on standard output (`--format json`). A
//! `JUnit` XML file is written at the end with `--junit FILE`.
//!
//! JSON events, one per line (docs/tools/jacket-test.md lists every field):
//! `{"type":"run","event":"started","tests":N}`,
//! `{"type":"test","event":"started","name":...}`, `{"type":"test","event":"slow",...}`,
//! `{"type":"test","event":"passed"|"failed"|"skipped"|"timed_out"|"xfail"|"xpass"|"flaky",...}`,
//! `{"type":"run","event":"finished",...}`, and with `--list` one `{"type":"list",...}` per test.
//! `type` names the kind of item (`test`; `bench` is reserved for benchmark results).
use super::{Case, Kind, Outcome, Status};
use crate::cli::OutputFormat;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct Reporter {
    format: OutputFormat,
    junit: Option<PathBuf>,
    cwd: PathBuf,
    interp: bool,
    color: bool,
    capture: bool,
    quiet: bool,
    sources: BTreeMap<String, Vec<String>>,
}

fn label(status: Status) -> &'static str {
    match status {
        Status::Pass => "PASS",
        Status::Fail => "FAIL",
        Status::Skip => "SKIP",
        Status::Timeout => "TIMEOUT",
        Status::XFail => "XFAIL",
        Status::XPass => "XPASS",
        Status::Flaky => "FLAKY",
    }
}

fn event(status: Status) -> &'static str {
    match status {
        Status::Pass => "passed",
        Status::Fail => "failed",
        Status::Skip => "skipped",
        Status::Timeout => "timed_out",
        Status::XFail => "xfail",
        Status::XPass => "xpass",
        Status::Flaky => "flaky",
    }
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn indent(text: &str, by: &str) -> String {
    text.lines()
        .map(|l| format!("{by}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// `'name'` for a shell, with `'` escaped.
fn quoted(name: &str) -> String {
    format!("'{}'", name.replace('\'', "'\\''"))
}

/// What `--exact` is given to run a failed test again: one to three failed subtests by name,
/// more of them (or none) the whole test.
fn rerun_names(case: &Case, failed_subtests: &[&str]) -> String {
    if !(1..=3).contains(&failed_subtests.len()) {
        return quoted(&case.name);
    }
    failed_subtests
        .iter()
        .map(|sub| match sub.strip_prefix(case.local.as_str()) {
            Some(rest) => quoted(&format!("{}{rest}", case.name)),
            None => quoted(&case.name),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            // Control characters other than tab and line breaks are not allowed in XML 1.0.
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => out.push('?'),
            c => out.push(c),
        }
    }
    out
}

impl Reporter {
    /// Whether results are text for people (so a coverage table can share standard output).
    pub fn is_human(&self) -> bool {
        self.format == OutputFormat::Human
    }

    pub fn new(
        format: OutputFormat,
        junit: Option<PathBuf>,
        cwd: &Path,
        interp: bool,
        capture: bool,
        quiet: bool,
    ) -> Reporter {
        use std::io::IsTerminal;
        let color = std::io::stderr().is_terminal() && jacket_config::env::NO_COLOR.os().is_none();
        Reporter {
            format,
            junit,
            cwd: cwd.to_path_buf(),
            interp,
            color,
            capture,
            quiet,
            sources: BTreeMap::new(),
        }
    }

    fn json(&self, value: &Value) {
        let mut out = std::io::stdout().lock();
        let _ = writeln!(out, "{value}");
    }

    fn shown(&self, file: &str) -> String {
        let path = Path::new(file);
        path.strip_prefix(&self.cwd)
            .unwrap_or(path)
            .display()
            .to_string()
    }

    fn paint(&self, code: &str, text: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{text}\x1b[0m")
        } else {
            text.to_string()
        }
    }

    fn status_word(&self, status: Status) -> String {
        let word = format!("{:>12}", label(status));
        match status {
            Status::Pass | Status::XFail => self.paint("1;32", &word),
            Status::Skip => self.paint("1;33", &word),
            Status::Flaky => self.paint("1;35", &word),
            _ => self.paint("1;31", &word),
        }
    }

    pub fn list(&self, cases: &[Case]) {
        for case in cases {
            match self.format {
                OutputFormat::Json => self.json(&json!({
                    "type": "list",
                    "kind": case.kind.as_str(),
                    "name": case.name,
                    "file": case.file,
                    "line": case.line,
                    "notes": case.notes,
                })),
                OutputFormat::Human => println!("{}: {}", case.name, case.kind.as_str()),
            }
            for sub in &case.cases {
                match self.format {
                    OutputFormat::Json => self.json(&json!({
                        "type": "list",
                        "kind": "subtest",
                        "name": format!("{}/{sub}", case.name),
                        "file": case.file,
                        "line": case.line,
                        "notes": case.notes,
                    })),
                    OutputFormat::Human => println!("{}/{sub}: subtest", case.name),
                }
            }
        }
    }

    pub fn run_started(&self, tests: usize) {
        match self.format {
            OutputFormat::Json => {
                self.json(&json!({"type": "run", "event": "started", "tests": tests}));
            }
            OutputFormat::Human => {
                let word = self.paint("1;32", &format!("{:>12}", "Starting"));
                eprintln!(
                    "{word} {tests} test{}",
                    if tests == 1 {
                        ""
                    } else {
                        "s"
                    }
                );
            }
        }
    }

    pub fn started(&self, case: &Case) {
        if self.format == OutputFormat::Json {
            self.json(&json!({"type": "test", "event": "started", "name": case.name, "kind": case.kind.as_str()}));
        }
    }

    pub fn slow(&self, case: &Case, elapsed: Duration) {
        match self.format {
            OutputFormat::Json => self.json(&json!({
                "type": "test", "event": "slow", "name": case.name, "seconds": elapsed.as_secs_f64(),
            })),
            OutputFormat::Human => {
                let word = self.paint("1;33", &format!("{:>12}", "SLOW"));
                eprintln!("{word} [>{:>7.3}s] {}", elapsed.as_secs_f64(), case.name);
            }
        }
    }

    pub fn finished(&self, case: &Case, outcome: &Outcome) {
        match self.format {
            OutputFormat::Json => self.json(&self.outcome_json(case, outcome)),
            OutputFormat::Human => {
                if self.quiet
                    && matches!(outcome.status, Status::Pass | Status::Skip | Status::XFail)
                {
                    return;
                }
                let time = if outcome.status == Status::Skip {
                    " ".repeat(11)
                } else {
                    format!("[{:>8.3}s]", outcome.duration.as_secs_f64())
                };
                let mut line = format!("{} {time} {}", self.status_word(outcome.status), case.name);
                if let Some(reason) = &outcome.skip_reason
                    && !reason.is_empty()
                {
                    line.push_str(&format!(" ({reason})"));
                }
                if outcome.status == Status::Flaky {
                    line.push_str(&format!(" (passed on try {})", outcome.attempts));
                }
                eprintln!("{line}");
            }
        }
    }

    fn outcome_json(&self, case: &Case, outcome: &Outcome) -> Value {
        json!({
            "type": "test",
            "event": event(outcome.status),
            "name": case.name,
            "kind": case.kind.as_str(),
            "file": case.file,
            "line": case.line,
            "duration": outcome.duration.as_secs_f64(),
            "attempts": outcome.attempts,
            "reason": outcome.skip_reason,
            "failures": outcome.failures.iter().map(|f| json!({
                "test": f.test, "file": f.file, "line": f.line, "message": f.message,
            })).collect::<Vec<_>>(),
            "problems": outcome.problems,
            "subtests": outcome.subtests.iter().map(|s| json!({
                "name": s.name, "status": s.status, "duration": s.duration.as_secs_f64(),
            })).collect::<Vec<_>>(),
            "stdout": text(&outcome.stdout),
            "stderr": text(&outcome.stderr),
            "temp_dir": outcome.kept_temp,
        })
    }

    /// The source line `line` of `file`, trimmed, when the file can be read.
    fn source_line(&mut self, file: &str, line: u32) -> Option<String> {
        let lines = self.sources.entry(file.to_string()).or_insert_with(|| {
            std::fs::read_to_string(file)
                .map(|t| t.lines().map(String::from).collect())
                .unwrap_or_default()
        });
        let text = lines.get((line as usize).checked_sub(1)?)?.trim();
        (!text.is_empty()).then(|| text.to_string())
    }

    fn details(&mut self, case: &Case, outcome: &Outcome) -> String {
        let mut out = String::new();
        for failure in &outcome.failures {
            let place = format!("{}:{}", self.shown(&failure.file), failure.line);
            let within = if failure.test == case.local {
                String::new()
            } else {
                format!(" (in {})", failure.test)
            };
            out.push_str(&format!(
                "  {place}{within}:\n{}\n",
                indent(&failure.message, "    ")
            ));
            if let Some(source) = self.source_line(&failure.file, failure.line) {
                out.push_str(&format!(
                    "    {} {source}\n",
                    self.paint("2", &format!("{} |", failure.line))
                ));
            }
        }
        for problem in &outcome.problems {
            out.push_str(&format!("{}\n", indent(problem, "  ")));
        }
        let failed_subtests: Vec<&str> = outcome
            .subtests
            .iter()
            .filter(|s| s.status == "fail")
            .map(|s| s.name.as_str())
            .collect();
        if !failed_subtests.is_empty() {
            out.push_str(&format!(
                "  failed subtests: {}\n",
                failed_subtests.join(", ")
            ));
        }
        if self.capture {
            for (name, bytes) in [("stdout", &outcome.stdout), ("stderr", &outcome.stderr)] {
                let text = text(bytes);
                if !text.trim().is_empty() {
                    out.push_str(&format!("  {name}:\n{}\n", indent(text.trim_end(), "    ")));
                }
            }
        }
        if let Some(temp) = &outcome.kept_temp {
            out.push_str(&format!("  temporary folder kept: {}\n", temp.display()));
        }
        let interp = if self.interp {
            " --interp"
        } else {
            ""
        };
        let reruns = rerun_names(case, &failed_subtests);
        out.push_str(&format!("  rerun: jacket test{interp} --exact {reruns}\n"));
        out
    }

    pub fn summary(
        &mut self,
        cases: &[Case],
        outcomes: &[Outcome],
        not_run: usize,
        elapsed: Duration,
        flaky_fails: bool,
    ) {
        let count = |s: Status| outcomes.iter().filter(|o| o.status == s).count();
        let counts = [
            ("passed", count(Status::Pass)),
            ("failed", count(Status::Fail)),
            ("timed out", count(Status::Timeout)),
            ("flaky", count(Status::Flaky)),
            ("xfail", count(Status::XFail)),
            ("xpass", count(Status::XPass)),
            ("skipped", count(Status::Skip)),
            ("not run", not_run),
        ];
        if let Some(path) = self.junit.clone()
            && let Err(error) = std::fs::write(&path, junit(cases, outcomes, elapsed))
        {
            eprintln!("jacket: warning: cannot write {}: {error}", path.display());
        }
        if self.format == OutputFormat::Json {
            let mut value =
                json!({"type": "run", "event": "finished", "duration": elapsed.as_secs_f64()});
            for (name, n) in counts {
                value[name.replace(' ', "_")] = json!(n);
            }
            self.json(&value);
            return;
        }
        let failing: Vec<&Outcome> = outcomes
            .iter()
            .filter(|o| o.status.fails(flaky_fails) || o.status == Status::Flaky)
            .collect();
        let missing = super::prerequisites::missing(&failing);
        if !failing.is_empty() {
            eprintln!("\nfailures:");
            for outcome in failing {
                let case = &cases[outcome.case];
                let title = self.paint(
                    "1",
                    &format!("--- {} ({}) ---", case.name, label(outcome.status)),
                );
                let details = self.details(case, outcome);
                eprintln!("\n{title}\n{details}");
            }
        }
        if !missing.is_empty() {
            eprintln!("\nmissing build outputs (these explain failures above):");
            for line in missing {
                eprintln!("  {line}");
            }
        }
        let mut slowest: Vec<&Outcome> = outcomes
            .iter()
            .filter(|o| o.duration >= Duration::from_secs(1))
            .collect();
        slowest.sort_by_key(|a| std::cmp::Reverse(a.duration));
        if !slowest.is_empty() {
            eprintln!("slowest:");
            for outcome in slowest.iter().take(5) {
                eprintln!(
                    "  {:>8.3}s {}",
                    outcome.duration.as_secs_f64(),
                    cases[outcome.case].name
                );
            }
        }
        let ran = outcomes.len() - count(Status::Skip);
        let parts: Vec<String> = counts
            .iter()
            .filter(|(name, n)| *n > 0 || *name == "passed")
            .map(|(name, n)| format!("{n} {name}"))
            .collect();
        let word = self.paint("1;32", &format!("{:>12}", "Summary"));
        eprintln!(
            "{word} [{:>8.3}s] {ran} test{} run: {}",
            elapsed.as_secs_f64(),
            if ran == 1 {
                ""
            } else {
                "s"
            },
            parts.join(", ")
        );
    }
}

/// The run as `JUnit` XML: one suite per root, one case per test.
fn junit(cases: &[Case], outcomes: &[Outcome], elapsed: Duration) -> String {
    let failures = outcomes
        .iter()
        .filter(|o| matches!(o.status, Status::Fail | Status::Timeout | Status::XPass))
        .count();
    let skipped = outcomes.iter().filter(|o| o.status == Status::Skip).count();
    let mut out = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites name=\"jacket test\" tests=\"{}\" failures=\"{failures}\" skipped=\"{skipped}\" time=\"{:.3}\">\n",
        outcomes.len(),
        elapsed.as_secs_f64()
    );
    let mut by_root: Vec<&Outcome> = outcomes.iter().collect();
    by_root.sort_by_key(|o| (cases[o.case].root, o.case));
    let mut current = None;
    for outcome in by_root {
        let case = &cases[outcome.case];
        if current != Some(case.root) {
            if current.is_some() {
                out.push_str("  </testsuite>\n");
            }
            current = Some(case.root);
            let suite = case.name.split_once("::").map_or("tests", |(root, _)| root);
            out.push_str(&format!("  <testsuite name=\"{}\">\n", xml(suite)));
        }
        let class = match case.kind {
            Kind::Test | Kind::Bench => "test",
            Kind::Golden => "golden",
            Kind::CompileFail => "compile_fail",
            Kind::Doc => "doc",
        };
        out.push_str(&format!(
            "    <testcase name=\"{}\" classname=\"{class}\" file=\"{}\" line=\"{}\" time=\"{:.3}\"",
            xml(&case.name),
            xml(&case.file.to_string_lossy()),
            case.line,
            outcome.duration.as_secs_f64()
        ));
        let mut body = String::new();
        let message: Vec<String> = outcome
            .failures
            .iter()
            .map(|f| format!("{}:{}: {}", f.file, f.line, f.message))
            .chain(outcome.problems.iter().cloned())
            .collect();
        match outcome.status {
            Status::Fail | Status::XPass => body.push_str(&format!(
                "      <failure message=\"{}\">{}</failure>\n",
                xml(message.first().map_or("failed", String::as_str)),
                xml(&message.join("\n"))
            )),
            Status::Timeout => body.push_str("      <failure message=\"timed out\"/>\n"),
            Status::Skip => body.push_str(&format!(
                "      <skipped message=\"{}\"/>\n",
                xml(outcome.skip_reason.as_deref().unwrap_or(""))
            )),
            _ => {}
        }
        for (tag, bytes) in [
            ("system-out", &outcome.stdout),
            ("system-err", &outcome.stderr),
        ] {
            if !bytes.is_empty() {
                body.push_str(&format!("      <{tag}>{}</{tag}>\n", xml(&text(bytes))));
            }
        }
        if body.is_empty() {
            out.push_str("/>\n");
        } else {
            out.push_str(&format!(">\n{body}    </testcase>\n"));
        }
    }
    if current.is_some() {
        out.push_str("  </testsuite>\n");
    }
    out.push_str("</testsuites>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::options::TestOptions;

    #[test]
    fn junit_lists_failures_and_skips() {
        let case = |name: &str| Case {
            name: name.into(),
            local: name.into(),
            kind: Kind::Test,
            root: 0,
            file: PathBuf::from("/t.jai"),
            line: 3,
            notes: Vec::new(),
            options: TestOptions::default(),
            run_as: name.into(),
            doc: None,
            cases: Vec::new(),
        };
        let cases = [case("a"), case("b<c>")];
        let mut failed = Outcome::new(1, Status::Fail);
        failed.problems.push("bad & worse".into());
        let mut skipped = Outcome::new(0, Status::Skip);
        skipped.skip_reason = Some("only on linux".into());
        let xml = junit(&cases, &[skipped, failed], Duration::from_millis(5));
        assert!(
            xml.contains("tests=\"2\" failures=\"1\" skipped=\"1\""),
            "{xml}"
        );
        assert!(xml.contains("name=\"b&lt;c&gt;\""));
        assert!(xml.contains("<failure message=\"bad &amp; worse\">"));
        assert!(xml.contains("<skipped message=\"only on linux\"/>"));
    }

    #[test]
    fn failed_subtests_are_rerun_by_name() {
        let case = Case {
            name: "Root::table".into(),
            local: "table".into(),
            kind: Kind::Test,
            root: 0,
            file: PathBuf::from("/t.jai"),
            line: 3,
            notes: Vec::new(),
            options: TestOptions::default(),
            run_as: "table".into(),
            doc: None,
            cases: Vec::new(),
        };
        assert_eq!(rerun_names(&case, &[]), "'Root::table'");
        assert_eq!(
            rerun_names(&case, &["table/a,1", "table/it's"]),
            "'Root::table/a,1' 'Root::table/it'\\''s'"
        );
        let many = ["table/a", "table/b", "table/c", "table/d"];
        assert_eq!(rerun_names(&case, &many), "'Root::table'");
    }
}
