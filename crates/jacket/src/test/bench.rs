//! `jacket bench`: measuring the `@bench` procedures. See docs/tools/jacket-bench.md.
//!
//! Discovery, the build, the filters and `--format` are `jacket test`'s. What differs is the
//! run: benchmarks run one after another (a parallel run would measure the machine's load), each
//! in its own process of the optimized program, started as `PROGRAM --bench NAME`. The program
//! warms up, measures and writes one `bench` record (JSON) to standard output; this file turns
//! the records into the table or the JSON document, saves and compares baselines, and decides
//! the exit status.
use super::build::{self, Toolchain};
use super::discover::Root;
use super::exec::{self, Demux, Invocation, Piece};
use super::options::Target;
use super::{Case, Settings};
use crate::cli::{BenchArgs, OutputFormat};
use serde_json::{Value, json};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

/// The version of the JSON document (`schema` in it).
const SCHEMA: u32 = 1;

/// A change smaller than this many times the larger MAD is noise.
const NOISE_FACTOR: f64 = 3.0;

/// What a run needs from the planning in `test/mod.rs`.
pub struct Run<'a> {
    pub cases: &'a [Case],
    pub roots: &'a [Root],
    pub built: &'a [Option<build::Built>],
    pub toolchain: &'a Toolchain,
    pub settings: &'a Settings,
    pub target: &'a Target,
    pub needs: &'a [String],
    pub project_dir: &'a Path,
    pub args: &'a BenchArgs,
}

/// What the program measured for one benchmark; times are nanoseconds per iteration.
#[derive(Clone, Debug, PartialEq)]
pub struct Measure {
    pub name: String,
    pub file: String,
    pub line: u32,
    pub samples: u64,
    pub iterations: u64,
    pub median: f64,
    pub mad: f64,
    pub p5: f64,
    pub p95: f64,
    pub min: f64,
    pub max: f64,
    pub bytes: u64,
    pub items: u64,
    pub allocs: f64,
    pub alloc_bytes: f64,
}

impl Measure {
    fn from_record(text: &str, case: &Case) -> Result<Measure, String> {
        let value: Value = serde_json::from_str(text).map_err(|e| format!("bad record: {e}"))?;
        let number = |key: &str| {
            value[key]
                .as_f64()
                .ok_or_else(|| format!("the record has no `{key}`"))
        };
        Ok(Measure {
            name: case.name.clone(),
            file: case.file.display().to_string(),
            line: case.line,
            samples: number("samples")? as u64,
            iterations: number("iterations")? as u64,
            median: number("median_ns")?,
            mad: number("mad_ns")?,
            p5: number("p5_ns")?,
            p95: number("p95_ns")?,
            min: number("min_ns")?,
            max: number("max_ns")?,
            bytes: number("bytes_per_iteration")? as u64,
            items: number("items_per_iteration")? as u64,
            allocs: number("allocs_per_iteration")?,
            alloc_bytes: number("alloc_bytes_per_iteration")?,
        })
    }

    /// Megabytes (10^6 bytes) per second, when the benchmark set its bytes.
    pub fn mb_per_s(&self) -> Option<f64> {
        (self.bytes > 0 && self.median > 0.0).then(|| self.bytes as f64 / self.median * 1000.0)
    }

    /// Items per second, when the benchmark set its items.
    pub fn items_per_s(&self) -> Option<f64> {
        (self.items > 0 && self.median > 0.0).then(|| self.items as f64 * 1e9 / self.median)
    }

    fn json(&self) -> Value {
        json!({
            "name": self.name,
            "file": self.file,
            "line": self.line,
            "samples": self.samples,
            "iterations": self.iterations,
            "median_ns": self.median,
            "mad_ns": self.mad,
            "p5_ns": self.p5,
            "p95_ns": self.p95,
            "min_ns": self.min,
            "max_ns": self.max,
            "bytes_per_iteration": self.bytes,
            "items_per_iteration": self.items,
            "mb_per_s": self.mb_per_s(),
            "items_per_s": self.items_per_s(),
            "allocs_per_iteration": self.allocs,
            "alloc_bytes_per_iteration": self.alloc_bytes,
        })
    }
}

/// How a benchmark compares with the same one in a baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub baseline_median: f64,
    /// Positive is slower.
    pub percent: f64,
    /// Whether the difference is within `NOISE_FACTOR` times the larger MAD of the two runs.
    pub noise: bool,
}

impl Change {
    fn between(baseline: &Measure, now: &Measure) -> Option<Change> {
        if baseline.median <= 0.0 {
            return None;
        }
        let difference = now.median - baseline.median;
        Some(Change {
            baseline_median: baseline.median,
            percent: difference / baseline.median * 100.0,
            noise: difference.abs() <= NOISE_FACTOR * baseline.mad.max(now.mad),
        })
    }

    /// Slower than `limit` percent, and not by noise.
    pub fn regressed(&self, limit: f64) -> bool {
        self.percent > limit && !self.noise
    }
}

/// The saved results of an earlier run.
fn read_baseline(path: &Path) -> Result<Vec<Measure>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read the baseline {}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("the baseline {} is not JSON: {e}", path.display()))?;
    let results = value["results"]
        .as_array()
        .ok_or_else(|| format!("the baseline {} has no `results`", path.display()))?;
    results
        .iter()
        .map(|result| {
            let number = |key: &str| {
                result[key]
                    .as_f64()
                    .ok_or_else(|| format!("a baseline result has no `{key}`"))
            };
            Ok(Measure {
                name: result["name"]
                    .as_str()
                    .ok_or("a baseline result has no `name`")?
                    .to_string(),
                file: String::new(),
                line: 0,
                samples: 0,
                iterations: 0,
                median: number("median_ns")?,
                mad: number("mad_ns")?,
                p5: 0.0,
                p95: 0.0,
                min: 0.0,
                max: 0.0,
                bytes: 0,
                items: 0,
                allocs: 0.0,
                alloc_bytes: 0.0,
            })
        })
        .collect()
}

pub fn baseline_path(project_dir: &Path, name: &str) -> PathBuf {
    project_dir
        .join("target")
        .join("bench-baselines")
        .join(format!("{name}.json"))
}

/// Runs the benchmarks of the plan and reports them.
pub fn run(run: &Run) -> ExitCode {
    let args = run.args;
    let baseline = match &args.baseline {
        Some(name) => match read_baseline(&baseline_path(run.project_dir, name)) {
            Ok(results) => Some(results),
            Err(error) => return super::fail(error),
        },
        None => None,
    };
    let mut measures = Vec::new();
    let mut failed = false;
    for case in run.cases {
        if let Some(reason) = case.options.skip_reason(run.target, run.needs) {
            eprintln!("        SKIP {} ({reason})", case.name);
            continue;
        }
        eprintln!("     Running {}", case.name);
        match measure(run, case) {
            Ok(measure) => measures.push(measure),
            Err(error) => {
                eprintln!("jacket: error: {}: {error}", case.name);
                failed = true;
            }
        }
    }
    let changes: Vec<Option<Change>> = measures
        .iter()
        .map(|now| {
            baseline
                .as_ref()
                .and_then(|b| b.iter().find(|old| old.name == now.name))
                .and_then(|old| Change::between(old, now))
        })
        .collect();
    let document = document(&measures, &changes, run);
    match args.test.format {
        OutputFormat::Json => println!("{document:#}"),
        OutputFormat::Human => print!("{}", table(&measures, &changes, baseline.is_some())),
    }
    if let Some(name) = &args.save_baseline {
        let path = baseline_path(run.project_dir, name);
        let saved = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|()| std::fs::write(&path, format!("{document:#}\n")));
        match saved {
            Ok(()) => eprintln!("jacket: saved the baseline {}", path.display()),
            Err(error) => return super::fail(format!("cannot write {}: {error}", path.display())),
        }
    }
    if let (Some(limit), Some(_)) = (args.max_regression, &baseline) {
        for (measure, change) in measures.iter().zip(&changes) {
            if let Some(change) = change.as_ref().filter(|c| c.regressed(limit)) {
                eprintln!(
                    "jacket: regression: {} is {:+.1}% slower than the baseline (limit {limit}%)",
                    measure.name, change.percent
                );
                failed = true;
            }
        }
    }
    if failed {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Runs one benchmark in a process of its own.
fn measure(run: &Run, case: &Case) -> Result<Measure, String> {
    let built = run.built[case.root].as_ref().ok_or("it was not built")?;
    let (program, mut words) = build::program_and_args(&built.program, &run.toolchain.runner);
    words.extend(["--bench".to_string(), case.local.clone()]);
    for (flag, value) in [
        ("--samples", run.args.samples),
        ("--warmup", run.args.warmup),
        ("--sample-ms", run.args.sample_ms),
    ] {
        if let Some(value) = value {
            words.extend([flag.to_string(), value.to_string()]);
        }
    }
    let invocation = Invocation {
        program,
        args: words,
        cwd: run.roots[run.root_of(case)].dir.clone(),
        env: Vec::new(),
    };
    let limit = case.options.timeout.unwrap_or(run.settings.timeout);
    let (stdout, stderr, status) =
        execute(&invocation, (limit > 0).then(|| Duration::from_secs(limit)))?;
    let mut demux = Demux::default();
    let mut pieces = demux.feed(&stdout);
    let _ = demux.finish();
    let record = pieces.drain(..).find_map(|piece| match piece {
        Piece::Record(fields) if fields.first().is_some_and(|f| f == "bench") => {
            fields.get(1).cloned()
        }
        _ => None,
    });
    match record {
        Some(text) => Measure::from_record(&text, case),
        None => Err(format!(
            "the program wrote no result (exit status {status})\n{}",
            String::from_utf8_lossy(&stderr).trim_end()
        )),
    }
}

impl Run<'_> {
    fn root_of(&self, case: &Case) -> usize {
        case.root.min(self.roots.len().saturating_sub(1))
    }
}

/// Runs the process to its end, or kills it after `limit`.
fn execute(
    invocation: &Invocation,
    limit: Option<Duration>,
) -> Result<(Vec<u8>, Vec<u8>, String), String> {
    let mut child = exec::spawn(invocation)
        .map_err(|e| format!("could not start `{}`: {e}", invocation.program.display()))?;
    let drain = |mut stream: Box<dyn Read + Send>| {
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes);
            bytes
        })
    };
    let out = drain(Box::new(child.stdout.take().expect("piped")));
    let err = drain(Box::new(child.stderr.take().expect("piped")));
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.to_string(),
            Ok(None) => {}
            Err(error) => return Err(format!("cannot wait for the program: {error}")),
        }
        if limit.is_some_and(|l| started.elapsed() > l) {
            exec::kill_tree(&mut child);
            let _ = child.wait();
            return Err(format!(
                "timed out after {}s (`@timeout(SECONDS)` or `--timeout` raises the limit)",
                started.elapsed().as_secs()
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Ok((
        out.join().unwrap_or_default(),
        err.join().unwrap_or_default(),
        status,
    ))
}

/// The JSON document of a run, which is also what a baseline file holds.
fn document(measures: &[Measure], changes: &[Option<Change>], run: &Run) -> Value {
    let results: Vec<Value> = measures
        .iter()
        .zip(changes)
        .map(|(measure, change)| {
            let mut value = measure.json();
            if let Some(change) = change {
                value["baseline"] = json!({
                    "median_ns": change.baseline_median,
                    "change_percent": change.percent,
                    "noise": change.noise,
                    "regression": run
                        .args
                        .max_regression
                        .map(|limit| change.regressed(limit)),
                });
            }
            value
        })
        .collect();
    json!({
        "schema": SCHEMA,
        "jacket": env!("CARGO_PKG_VERSION"),
        "os": run.target.os,
        "cpu": run.target.cpu,
        "results": results,
    })
}

/// A duration in nanoseconds as `812 ns`, `4.31 us`, `1.20 ms` or `2.50 s`.
pub fn time_text(ns: f64) -> String {
    let (value, unit) = if ns < 1e3 {
        (ns, "ns")
    } else if ns < 1e6 {
        (ns / 1e3, "us")
    } else if ns < 1e9 {
        (ns / 1e6, "ms")
    } else {
        (ns / 1e9, "s")
    };
    if value >= 100.0 {
        format!("{value:.0} {unit}")
    } else if value >= 10.0 {
        format!("{value:.1} {unit}")
    } else {
        format!("{value:.2} {unit}")
    }
}

fn rate_text(measure: &Measure) -> String {
    if let Some(mb) = measure.mb_per_s() {
        if mb >= 1000.0 {
            return format!("{:.2} GB/s", mb / 1000.0);
        }
        return format!("{mb:.1} MB/s");
    }
    if let Some(items) = measure.items_per_s() {
        return if items >= 1e9 {
            format!("{:.2} G/s", items / 1e9)
        } else if items >= 1e6 {
            format!("{:.2} M/s", items / 1e6)
        } else if items >= 1e3 {
            format!("{:.2} k/s", items / 1e3)
        } else {
            format!("{items:.1} /s")
        };
    }
    "-".to_string()
}

fn change_text(change: Option<&Change>) -> String {
    match change {
        Some(change) if change.noise => format!("{:+.1}% (noise)", change.percent),
        Some(change) => format!("{:+.1}%", change.percent),
        None => "new".to_string(),
    }
}

/// The table for people: a row per benchmark, columns aligned.
pub fn table(measures: &[Measure], changes: &[Option<Change>], compared: bool) -> String {
    let mut header = vec![
        "benchmark".to_string(),
        "median".to_string(),
        "± mad".to_string(),
        "throughput".to_string(),
        "allocs/iter".to_string(),
    ];
    if compared {
        header.push("change".to_string());
    }
    let mut rows = vec![header];
    for (measure, change) in measures.iter().zip(changes) {
        let mut row = vec![
            measure.name.clone(),
            time_text(measure.median),
            time_text(measure.mad),
            rate_text(measure),
            format!("{:.2}", measure.allocs),
        ];
        if compared {
            row.push(change_text(change.as_ref()));
        }
        rows.push(row);
    }
    let columns = rows[0].len();
    let widths: Vec<usize> = (0..columns)
        .map(|c| rows.iter().map(|r| r[c].chars().count()).max().unwrap_or(0))
        .collect();
    let mut out = String::new();
    for row in &rows {
        let cells: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(c, cell)| {
                let pad = widths[c] - cell.chars().count();
                if c == 0 {
                    format!("{cell}{}", " ".repeat(pad))
                } else {
                    format!("{}{cell}", " ".repeat(pad))
                }
            })
            .collect();
        out.push_str(cells.join("  ").trim_end());
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measure(name: &str, median: f64, mad: f64) -> Measure {
        Measure {
            name: name.into(),
            file: String::new(),
            line: 1,
            samples: 30,
            iterations: 1000,
            median,
            mad,
            p5: median,
            p95: median,
            min: median,
            max: median,
            bytes: 0,
            items: 0,
            allocs: 0.0,
            alloc_bytes: 0.0,
        }
    }

    #[test]
    fn a_slowdown_inside_the_noise_is_not_a_regression() {
        let old = measure("a", 100.0, 5.0);
        let noisy = Change::between(&old, &measure("a", 112.0, 5.0)).unwrap();
        assert!(noisy.noise);
        assert!(!noisy.regressed(5.0));
        let slower = Change::between(&old, &measure("a", 130.0, 5.0)).unwrap();
        assert!(!slower.noise);
        assert!(slower.regressed(10.0));
        assert!(!slower.regressed(40.0));
        let faster = Change::between(&old, &measure("a", 60.0, 1.0)).unwrap();
        assert!(!faster.regressed(0.0));
        assert!(faster.percent < 0.0);
    }

    #[test]
    fn throughput_comes_from_the_median_and_the_work_per_iteration() {
        let mut m = measure("a", 1000.0, 1.0);
        assert_eq!(m.mb_per_s(), None);
        m.bytes = 2000;
        assert_eq!(m.mb_per_s(), Some(2000.0));
        m.items = 5;
        assert_eq!(m.items_per_s(), Some(5_000_000.0));
    }

    #[test]
    fn times_use_three_significant_digits() {
        assert_eq!(time_text(812.0), "812 ns");
        assert_eq!(time_text(4310.0), "4.31 us");
        assert_eq!(time_text(12_300_000.0), "12.3 ms");
        assert_eq!(time_text(2.5e9), "2.50 s");
    }

    #[test]
    fn records_are_read_into_measures() {
        let case = super::super::tests::case("hash", super::super::Kind::Bench);
        let text = r#"{"name":"hash","samples":30,"iterations":4096,"median_ns":12.5,"mad_ns":0.25,"min_ns":12.0,"max_ns":14.0,"p5_ns":12.1,"p95_ns":13.5,"bytes_per_iteration":16,"items_per_iteration":0,"allocs_per_iteration":0.0,"alloc_bytes_per_iteration":0.0}"#;
        let measure = Measure::from_record(text, &case).unwrap();
        assert_eq!((measure.samples, measure.iterations), (30, 4096));
        assert_eq!(measure.median, 12.5);
        assert!(Measure::from_record("{}", &case).is_err());
    }

    #[test]
    fn the_table_marks_changes_and_aligns_columns() {
        let measures = [
            measure("short", 100.0, 1.0),
            measure("a_longer_name", 2000.0, 2.0),
        ];
        let changes = [
            Change::between(&measure("short", 80.0, 1.0), &measures[0]),
            None,
        ];
        let text = table(&measures, &changes, true);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[1].contains("+25.0%"));
        assert!(lines[2].ends_with("new"));
        assert!(lines[0].starts_with("benchmark"));
    }

    #[test]
    fn baselines_live_under_target() {
        assert_eq!(
            baseline_path(Path::new("/p"), "main"),
            PathBuf::from("/p/target/bench-baselines/main.json")
        );
    }
}
