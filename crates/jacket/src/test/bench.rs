//! `jacket bench`: measuring the `@bench` procedures. See docs/tools/jacket-bench.md.
//!
//! Discovery, the build, the filters and `--format` are `jacket test`'s. What differs is the
//! run: benchmarks run one after another (a parallel run would measure the machine's load), each
//! in its own process of the optimized program, started as `PROGRAM --bench NAME`. The program
//! warms up, takes the samples and writes one `bench` record (JSON) with the raw measurements to
//! standard output. The statistics (`bench_stats.rs`), the report, the baselines and the exit
//! status are made here, not in the program.
use super::bench_stats::{self as stats, Analysis, Comparison, Interval, Rng, Samples, Verdict};
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
const SCHEMA: u32 = 2;

/// The seed of the bootstrap, so the same measurements always give the same report.
const SEED: u64 = 0x6a61_636b_6574;

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

/// What the program measured for one benchmark, and what the statistics make of it; times are
/// nanoseconds per iteration.
#[derive(Clone, Debug, PartialEq)]
pub struct Measure {
    pub name: String,
    pub file: String,
    pub line: u32,
    /// `linear` (sample k runs k * step iterations) or `flat` (a bench too slow for that).
    pub mode: String,
    pub step: u64,
    /// How many times the samples were taken: a take spoiled by other load is taken again.
    pub attempts: u64,
    /// The core the program pinned itself to, where the system allows it.
    pub cpu: Option<u64>,
    pub samples: Samples,
    pub analysis: Analysis,
    pub bytes: u64,
    pub items: u64,
    pub allocs: f64,
    pub alloc_bytes: f64,
}

/// Reads a list of whole numbers as floats.
fn number_list(value: &Value, key: &str) -> Result<Vec<f64>, String> {
    value[key]
        .as_array()
        .ok_or_else(|| format!("the record has no `{key}`"))?
        .iter()
        .map(|v| {
            v.as_f64()
                .ok_or_else(|| format!("`{key}` holds a non-number"))
        })
        .collect()
}

fn read_samples(value: &Value) -> Result<Samples, String> {
    let samples = Samples {
        iterations: number_list(value, "sample_iterations")?,
        nanoseconds: number_list(value, "sample_ns")?,
    };
    if samples.len() < 2 || samples.len() != samples.nanoseconds.len() {
        return Err("the record has fewer than two samples, or the lists differ in length".into());
    }
    if samples.iterations.iter().any(|&i| i < 1.0) {
        return Err("a sample ran no iterations".into());
    }
    Ok(samples)
}

impl Measure {
    fn from_record(text: &str, case: &Case, resamples: usize) -> Result<Measure, String> {
        let value: Value = serde_json::from_str(text).map_err(|e| format!("bad record: {e}"))?;
        let number = |key: &str| {
            value[key]
                .as_f64()
                .ok_or_else(|| format!("the record has no `{key}`"))
        };
        let samples = read_samples(&value)?;
        let analysis = stats::analyze(&samples, resamples, &mut Rng::new(SEED));
        Ok(Measure {
            name: case.name.clone(),
            file: case.file.display().to_string(),
            line: case.line,
            mode: value["mode"].as_str().unwrap_or("linear").to_string(),
            step: number("step")? as u64,
            attempts: value["attempts"].as_u64().unwrap_or(1),
            cpu: value["cpu"].as_i64().and_then(|c| u64::try_from(c).ok()),
            samples,
            analysis,
            bytes: number("bytes_per_iteration")? as u64,
            items: number("items_per_iteration")? as u64,
            allocs: number("allocs_per_iteration")?,
            alloc_bytes: number("alloc_bytes_per_iteration")?,
        })
    }

    /// Megabytes (10^6 bytes) per second, when the benchmark set its bytes.
    pub fn mb_per_s(&self) -> Option<f64> {
        let time = self.analysis.time.estimate;
        (self.bytes > 0 && time > 0.0).then(|| self.bytes as f64 / time * 1000.0)
    }

    /// Items per second, when the benchmark set its items.
    pub fn items_per_s(&self) -> Option<f64> {
        let time = self.analysis.time.estimate;
        (self.items > 0 && time > 0.0).then(|| self.items as f64 * 1e9 / time)
    }

    fn json(&self) -> Value {
        let a = &self.analysis;
        json!({
            "name": self.name,
            "file": self.file,
            "line": self.line,
            "mode": self.mode,
            "samples": self.samples.len(),
            "step": self.step,
            "attempts": self.attempts,
            "cpu": self.cpu,
            "estimator": a.estimator.name(),
            "time_ns": interval_json(a.time),
            "mean_ns": interval_json(a.mean),
            "median_ns": interval_json(a.median),
            "mad_ns": a.mad,
            "r_squared": a.r_squared,
            "overhead_ns": a.overhead,
            "outliers": {
                "low_severe": a.outliers.low_severe,
                "low_mild": a.outliers.low_mild,
                "high_mild": a.outliers.high_mild,
                "high_severe": a.outliers.high_severe,
            },
            "bytes_per_iteration": self.bytes,
            "items_per_iteration": self.items,
            "mb_per_s": self.mb_per_s(),
            "items_per_s": self.items_per_s(),
            "allocs_per_iteration": self.allocs,
            "alloc_bytes_per_iteration": self.alloc_bytes,
            "sample_iterations": self.samples.iterations,
            "sample_ns": self.samples.nanoseconds,
        })
    }
}

fn interval_json(i: Interval) -> Value {
    json!({ "low": i.low, "estimate": i.estimate, "high": i.high })
}

/// How a benchmark compares with the same one in a baseline.
#[derive(Clone, Debug, PartialEq)]
pub struct Change {
    pub baseline_time: f64,
    pub comparison: Comparison,
}

impl Change {
    /// Whether this is a regression past `limit` percent: slower by that much, and a change the
    /// statistics stand behind.
    pub fn regressed(&self, limit: f64) -> bool {
        self.comparison.verdict == Verdict::Regressed
            && self.comparison.change.estimate * 100.0 > limit
    }
}

/// The samples of the benchmarks of an earlier run.
fn read_baseline(path: &Path) -> Result<Vec<(String, Samples)>, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|e| format!("cannot read the baseline {}: {e}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|e| format!("the baseline {} is not JSON: {e}", path.display()))?;
    if value["schema"].as_u64() != Some(u64::from(SCHEMA)) {
        return Err(format!(
            "the baseline {} was saved by an older jacket (schema {}, this is {SCHEMA}); save it again",
            path.display(),
            value["schema"]
        ));
    }
    let results = value["results"]
        .as_array()
        .ok_or_else(|| format!("the baseline {} has no `results`", path.display()))?;
    results
        .iter()
        .map(|result| {
            let name = result["name"]
                .as_str()
                .ok_or("a baseline result has no `name`")?
                .to_string();
            Ok((name, read_samples(result)?))
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
    let resamples = args.resamples as usize;
    let noise = args.noise_threshold / 100.0;
    let mut failed = false;
    let mut results: Vec<(Measure, Option<Change>)> = Vec::new();
    for case in run.cases {
        if let Some(reason) = case.options.skip_reason(run.target, run.needs) {
            eprintln!("        SKIP {} ({reason})", case.name);
            continue;
        }
        eprintln!("     Running {}", case.name);
        match measure(run, case) {
            Ok(measure) => {
                let change = baseline
                    .as_ref()
                    .and_then(|b| b.iter().find(|(name, _)| *name == measure.name))
                    .and_then(|(_, old)| {
                        let comparison = stats::compare(
                            &measure.samples,
                            old,
                            resamples,
                            noise,
                            args.significance,
                            &mut Rng::new(SEED),
                        )?;
                        let baseline_time =
                            stats::analyze(old, 0, &mut Rng::new(SEED)).time.estimate;
                        Some(Change {
                            baseline_time,
                            comparison,
                        })
                    });
                if args.test.format == OutputFormat::Human {
                    print!(
                        "{}",
                        report(&measure, change.as_ref(), baseline.is_some(), args)
                    );
                }
                results.push((measure, change));
            }
            Err(error) => {
                eprintln!("jacket: error: {}: {error}", case.name);
                failed = true;
            }
        }
    }
    let document = document(&results, run);
    if args.test.format == OutputFormat::Json {
        println!("{document:#}");
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
        for (measure, change) in &results {
            if let Some(change) = change.as_ref().filter(|c| c.regressed(limit)) {
                eprintln!(
                    "jacket: regression: {} is {:+.1}% slower than the baseline (limit {limit}%)",
                    measure.name,
                    change.comparison.change.estimate * 100.0
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
        ("--measure-ms", run.args.measure_ms),
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
        Some(text) => Measure::from_record(&text, case, run.args.resamples as usize),
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
fn document(results: &[(Measure, Option<Change>)], run: &Run) -> Value {
    let results: Vec<Value> = results
        .iter()
        .map(|(measure, change)| {
            let mut value = measure.json();
            if let Some(change) = change {
                let c = &change.comparison;
                value["baseline"] = json!({
                    "time_ns": change.baseline_time,
                    "change": interval_json(c.change),
                    "change_percent": c.change.estimate * 100.0,
                    "p_value": c.p_value,
                    "verdict": c.verdict.name(),
                    "regression": run.args.max_regression.map(|limit| change.regressed(limit)),
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
        "confidence_level": stats::CONFIDENCE,
        "resamples": run.args.resamples,
        "noise_threshold_percent": run.args.noise_threshold,
        "significance_level": run.args.significance,
        "results": results,
    })
}

/// The unit that shows `ns` with a few digits, and the divisor to get there.
fn unit_of(ns: f64) -> (f64, &'static str) {
    if ns < 1e3 {
        (1.0, "ns")
    } else if ns < 1e6 {
        (1e3, "µs")
    } else if ns < 1e9 {
        (1e6, "ms")
    } else {
        (1e9, "s")
    }
}

/// Four significant digits.
fn digits(value: f64) -> String {
    if value >= 1000.0 {
        format!("{value:.0}")
    } else if value >= 100.0 {
        format!("{value:.1}")
    } else if value >= 10.0 {
        format!("{value:.2}")
    } else {
        format!("{value:.3}")
    }
}

/// `[low estimate high]` of a time, in the unit of the estimate. Negative times (a fit through
/// noise around zero) show as zero.
pub fn time_triplet(i: Interval) -> String {
    let (divisor, unit) = unit_of(i.estimate.max(0.0));
    let at = |v: f64| digits(v.max(0.0) / divisor);
    format!("[{} {} {} {unit}]", at(i.low), at(i.estimate), at(i.high))
}

fn percent_triplet(i: Interval) -> String {
    format!(
        "[{:+.2}% {:+.2}% {:+.2}%]",
        i.low * 100.0,
        i.estimate * 100.0,
        i.high * 100.0
    )
}

/// `[low estimate high]` of a rate: a slower time is a lower rate, so the ends swap.
fn rate_triplet(measure: &Measure) -> Option<String> {
    let time = measure.analysis.time;
    if time.estimate <= 0.0 || time.low <= 0.0 || time.high <= 0.0 {
        return None;
    }
    let (work, scale): (f64, &[(f64, &str)]) = if measure.bytes > 0 {
        (
            measure.bytes as f64,
            &[(1e9, "GB/s"), (1e6, "MB/s"), (1e3, "kB/s"), (1.0, "B/s")],
        )
    } else if measure.items > 0 {
        (
            measure.items as f64,
            &[(1e9, "G/s"), (1e6, "M/s"), (1e3, "k/s"), (1.0, "/s")],
        )
    } else {
        return None;
    };
    let rate = |ns: f64| work / ns * 1e9;
    let (low, estimate, high) = (rate(time.high), rate(time.estimate), rate(time.low));
    let &(divisor, unit) = scale
        .iter()
        .find(|(d, _)| estimate >= *d)
        .unwrap_or(&scale[scale.len() - 1]);
    Some(format!(
        "[{} {} {} {unit}]",
        digits(low / divisor),
        digits(estimate / divisor),
        digits(high / divisor)
    ))
}

/// The text for people, in the manner of criterion: the time with its interval, the change
/// against the baseline, then the outliers.
pub fn report(
    measure: &Measure,
    change: Option<&Change>,
    compared: bool,
    args: &BenchArgs,
) -> String {
    const COLUMN: usize = 24;
    let a = &measure.analysis;
    let pad = " ".repeat(COLUMN);
    let mut out = String::new();
    if measure.name.chars().count() >= COLUMN {
        out.push_str(&format!("{}\n{pad}", measure.name));
    } else {
        out.push_str(&format!("{:<COLUMN$}", measure.name));
    }
    out.push_str(&format!("time:   {}\n", time_triplet(a.time)));
    if let Some(rate) = rate_triplet(measure) {
        out.push_str(&format!("{pad}thrpt:  {rate}\n"));
    }
    let mut details = Vec::new();
    if let Some(r) = a.r_squared {
        details.push(format!("R² {r:.4}"));
    }
    details.push(format!("median {}", time_triplet(a.median)));
    details.push(format!("allocs/iter {:.2}", measure.allocs));
    if measure.attempts > 1 {
        details.push(format!("{} attempts", measure.attempts));
    }
    out.push_str(&format!("{pad}{}\n", details.join("   ")));
    match change {
        Some(change) => {
            let c = &change.comparison;
            let relation = if c.p_value < args.significance {
                "<"
            } else {
                ">"
            };
            out.push_str(&format!(
                "{pad}change: {} (p = {:.2} {relation} {:.2})\n{pad}{}\n",
                percent_triplet(c.change),
                c.p_value,
                args.significance,
                c.verdict.sentence()
            ));
        }
        None if compared => out.push_str(&format!("{pad}change: new (not in the baseline)\n")),
        None => {}
    }
    let o = &a.outliers;
    if o.total() > 0 {
        let n = measure.samples.len();
        let share = |count: usize| count as f64 / n as f64 * 100.0;
        out.push_str(&format!(
            "Found {} outliers among {n} measurements ({:.2}%)\n",
            o.total(),
            share(o.total())
        ));
        for (count, kind) in [
            (o.low_severe, "low severe"),
            (o.low_mild, "low mild"),
            (o.high_mild, "high mild"),
            (o.high_severe, "high severe"),
        ] {
            if count > 0 {
                out.push_str(&format!("  {count} ({:.2}%) {kind}\n", share(count)));
            }
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn samples(per: f64, count: usize) -> Samples {
        Samples {
            iterations: (1..=count).map(|k| (k * 1000) as f64).collect(),
            nanoseconds: (1..=count)
                .map(|k| 400.0 + per * (k * 1000) as f64 + (k % 3) as f64)
                .collect(),
        }
    }

    fn measure(name: &str, per: f64) -> Measure {
        let samples = samples(per, 30);
        let analysis = stats::analyze(&samples, 500, &mut Rng::new(1));
        Measure {
            name: name.into(),
            file: String::new(),
            line: 1,
            mode: "linear".into(),
            step: 1000,
            attempts: 1,
            cpu: None,
            samples,
            analysis,
            bytes: 0,
            items: 0,
            allocs: 0.0,
            alloc_bytes: 0.0,
        }
    }

    fn args() -> BenchArgs {
        use clap::Parser;
        #[derive(Parser)]
        struct Wrapper {
            #[command(flatten)]
            bench: BenchArgs,
        }
        Wrapper::parse_from(["bench"]).bench
    }

    #[test]
    fn throughput_comes_from_the_time_and_the_work_per_iteration() {
        let mut m = measure("a", 1000.0);
        assert_eq!(m.mb_per_s(), None);
        m.bytes = 2000;
        assert!((m.mb_per_s().unwrap() - 2000.0).abs() < 1.0);
        m.items = 5;
        assert!((m.items_per_s().unwrap() - 5_000_000.0).abs() < 5_000.0);
    }

    #[test]
    fn times_show_four_digits_in_one_unit() {
        let t = |low, estimate, high| {
            time_triplet(Interval {
                low,
                estimate,
                high,
            })
        };
        assert_eq!(t(37.0, 37.5, 38.1), "[37.00 37.50 38.10 ns]");
        assert_eq!(t(4200.0, 4310.0, 4400.0), "[4.200 4.310 4.400 µs]");
        assert_eq!(t(-0.1, 0.0, 0.2), "[0.000 0.000 0.200 ns]");
        assert_eq!(t(2.4e9, 2.5e9, 2.6e9), "[2.400 2.500 2.600 s]");
    }

    #[test]
    fn the_report_has_the_time_the_outliers_and_the_change() {
        let mut m = measure("fnv1a", 40.0);
        m.bytes = 80;
        m.analysis.outliers.high_mild = 2;
        m.analysis.outliers.high_severe = 1;
        let old = samples(30.0, 30);
        let comparison =
            stats::compare(&m.samples, &old, 2000, 0.02, 0.05, &mut Rng::new(1)).unwrap();
        let change = Change {
            baseline_time: 30.0,
            comparison,
        };
        let text = report(&m, Some(&change), true, &args());
        let lines: Vec<&str> = text.lines().collect();
        assert!(
            lines[0].starts_with("fnv1a") && lines[0].contains("time:   [40."),
            "{text}"
        );
        assert!(text.contains("thrpt:  ["), "{text}");
        assert!(text.contains("change: [+33"), "{text}");
        assert!(text.contains("Performance has regressed."), "{text}");
        assert!(
            text.contains("Found 3 outliers among 30 measurements (10.00%)"),
            "{text}"
        );
        assert!(text.contains("  1 (3.33%) high severe"), "{text}");
        assert!(change.regressed(10.0) && !change.regressed(50.0));
    }

    #[test]
    fn a_long_name_gets_a_line_of_its_own() {
        let text = report(
            &measure("a_benchmark_with_a_very_long_name", 5.0),
            None,
            false,
            &args(),
        );
        assert!(text.starts_with("a_benchmark_with_a_very_long_name\n"));
        assert!(
            text.lines()
                .nth(1)
                .unwrap()
                .trim_start()
                .starts_with("time:")
        );
    }

    #[test]
    fn baselines_live_under_target() {
        assert_eq!(
            baseline_path(Path::new("/p"), "main"),
            PathBuf::from("/p/target/bench-baselines/main.json")
        );
    }

    #[test]
    fn records_are_read_into_measures() {
        let case = super::super::tests::case("hash", super::super::Kind::Bench);
        let text = r#"{"name":"hash","mode":"linear","samples":4,"step":100,"cpu":-1,"bytes_per_iteration":16,"items_per_iteration":0,"allocs_per_iteration":0.0,"alloc_bytes_per_iteration":0.0,"sample_iterations":[100,200,300,400],"sample_ns":[1010,2005,3020,4010]}"#;
        let measure = Measure::from_record(text, &case, 200).unwrap();
        assert_eq!((measure.samples.len(), measure.step), (4, 100));
        assert_eq!(measure.cpu, None);
        assert!((measure.analysis.time.estimate - 10.0).abs() < 0.1);
        assert!(Measure::from_record("{}", &case, 10).is_err());
        let single = text
            .replace("[100,200,300,400]", "[100]")
            .replace("[1010,2005,3020,4010]", "[1010]");
        assert!(Measure::from_record(&single, &case, 10).is_err());
    }

    #[test]
    fn an_old_baseline_is_refused_with_a_reason() {
        let dir = std::env::temp_dir().join(format!("jacket-bench-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("old.json");
        std::fs::write(&path, r#"{"schema":1,"results":[]}"#).unwrap();
        let error = read_baseline(&path).unwrap_err();
        assert!(error.contains("save it again"), "{error}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
