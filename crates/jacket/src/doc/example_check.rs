//! Testing the examples of a site: every one with a Run button is run before the site is written,
//! the way the browser runs it, so a Run button never opens on an error.
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// The most an example may run when it has no `timeout(n)` of its own.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

/// The Node script that runs programs in the browser bundle, as `run-worker.js` does.
const SCRIPT: &str = include_str!("run_examples.mjs");

#[cfg(test)]
#[path = "example_check_tests.rs"]
mod tests;

/// An example's program, as the Run button would send it.
#[derive(Clone, Debug)]
pub struct Program {
    /// `stdlib/String/module.jai:12`, where its doc comment is.
    pub label: String,
    pub text: String,
    /// Seconds, from `timeout(n)` on the fence.
    pub timeout: Option<u64>,
}

/// How the programs are run.
pub enum Engine<'a> {
    /// `jaic -interp -os wasm`: no browser bundle or Node needed.
    Interpreter {
        jaic: &'a Path,
        stdlib: &'a Path,
        import_dir: Vec<std::ffi::OsString>,
    },
    /// The bundle's own `engine.mjs` under Node, exactly the compiler the page loads.
    Browser { node: PathBuf, runtime: &'a Path },
}

pub struct Failure {
    pub label: String,
    pub message: String,
}

fn threads() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get().min(3))
}

fn scratch() -> Result<PathBuf, String> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let dir = std::env::temp_dir().join(format!(
        "jacket-doc-examples-{}-{nanos}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Runs `programs` and returns the ones that fail, in order.
pub fn run(engine: &Engine, programs: &[Program]) -> Result<Vec<Failure>, String> {
    let dir = scratch()?;
    let result = match engine {
        Engine::Interpreter {
            jaic,
            stdlib,
            import_dir,
        } => interpreted(jaic, stdlib, import_dir, programs, &dir),
        Engine::Browser {
            node,
            runtime,
        } => browser(node, runtime, programs, &dir),
    };
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn failures(programs: &[Program], mut outcomes: Vec<Option<String>>) -> Vec<Failure> {
    outcomes.resize(programs.len(), None);
    programs
        .iter()
        .zip(outcomes)
        .filter_map(|(p, message)| {
            message.map(|message| Failure {
                label: p.label.clone(),
                message,
            })
        })
        .collect()
}

fn interpreted(
    jaic: &Path,
    stdlib: &Path,
    import_dir: &[std::ffi::OsString],
    programs: &[Program],
    dir: &Path,
) -> Result<Vec<Failure>, String> {
    let next = AtomicUsize::new(0);
    let outcomes = Mutex::new(vec![None; programs.len()]);
    std::thread::scope(|scope| {
        for _ in 0..threads() {
            scope.spawn(|| {
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    let Some(program) = programs.get(i) else {
                        break;
                    };
                    let message = interpret(jaic, stdlib, import_dir, program, dir, i);
                    outcomes
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)[i] = message;
                }
            });
        }
    });
    let outcomes = outcomes
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(failures(programs, outcomes))
}

/// `None` when the program ran and exited 0, else what it printed.
fn interpret(
    jaic: &Path,
    stdlib: &Path,
    import_dir: &[std::ffi::OsString],
    program: &Program,
    dir: &Path,
    n: usize,
) -> Option<String> {
    let file = dir.join(format!("{n}.jai"));
    let out = dir.join(format!("{n}.out"));
    if let Err(e) = std::fs::write(&file, &program.text) {
        return Some(format!("cannot write {}: {e}", file.display()));
    }
    let log = match std::fs::File::create(&out).and_then(|f| Ok((f.try_clone()?, f))) {
        Ok(log) => log,
        Err(e) => return Some(format!("cannot write {}: {e}", out.display())),
    };
    let child = Command::new(jaic)
        .env(jacket_config::env::STDLIB.name, stdlib)
        .current_dir(dir)
        .arg(&file)
        .args(["-interp", "-quiet", "-os", "wasm"])
        .args(import_dir)
        .stdin(Stdio::null())
        .stdout(log.0)
        .stderr(log.1)
        .spawn();
    let mut child = match child {
        Ok(child) => child,
        Err(e) => return Some(format!("could not start `{}`: {e}", jaic.display())),
    };
    let limit = program.timeout.map_or(DEFAULT_TIMEOUT, Duration::from_secs);
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) if started.elapsed() > limit => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(e) => return Some(format!("could not wait for the example: {e}")),
        }
    };
    let printed = std::fs::read_to_string(&out).unwrap_or_default();
    let _ = std::fs::remove_file(&file);
    let _ = std::fs::remove_file(&out);
    match status {
        Some(status) if status.success() => None,
        Some(status) => Some(if printed.trim().is_empty() {
            format!("The example exited with {status}.")
        } else {
            printed.trim().to_owned()
        }),
        None => Some(format!("Stopped after {} seconds.", limit.as_secs())),
    }
}

fn browser(
    node: &Path,
    runtime: &Path,
    programs: &[Program],
    dir: &Path,
) -> Result<Vec<Failure>, String> {
    let script = dir.join("run_examples.mjs");
    std::fs::write(&script, SCRIPT)
        .map_err(|e| format!("cannot write {}: {e}", script.display()))?;
    let list = dir.join("programs.json");
    let json: Vec<_> = programs
        .iter()
        .map(|p| {
            serde_json::json!({
                "text": p.text,
                "timeoutSeconds": p.timeout.unwrap_or(DEFAULT_TIMEOUT.as_secs()).to_string(),
            })
        })
        .collect();
    std::fs::write(&list, serde_json::to_vec(&json).map_err(|e| e.to_string())?)
        .map_err(|e| format!("cannot write {}: {e}", list.display()))?;
    let shards = threads().min(programs.len().max(1));
    let outcomes = Mutex::new(vec![None; programs.len()]);
    let error: Mutex<Option<String>> = Mutex::new(None);
    std::thread::scope(|scope| {
        for shard in 0..shards {
            let (script, list, outcomes, error) = (&script, &list, &outcomes, &error);
            scope.spawn(move || {
                let output = Command::new(node)
                    .arg(script)
                    .arg(runtime)
                    .arg(list)
                    .arg(shard.to_string())
                    .arg(shards.to_string())
                    .stdin(Stdio::null())
                    .output();
                let output = match output {
                    Ok(output) => output,
                    Err(e) => {
                        *error
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner) =
                            Some(format!("could not start `{}`: {e}", node.display()));
                        return;
                    }
                };
                let mut seen = 0;
                for line in String::from_utf8_lossy(&output.stdout).lines() {
                    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
                        continue;
                    };
                    let (Some(i), Some(ok)) = (
                        value["i"].as_u64().and_then(|i| usize::try_from(i).ok()),
                        value["ok"].as_bool(),
                    ) else {
                        continue;
                    };
                    seen += 1;
                    if !ok && i < programs.len() {
                        outcomes
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)[i] =
                            Some(value["message"].as_str().unwrap_or("").to_owned());
                    }
                }
                let expected = (shard..programs.len()).step_by(shards).count();
                if seen < expected {
                    *error
                        .lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(format!(
                        "the browser compiler stopped after {seen} of {expected} examples:\n{}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    ));
                }
            });
        }
    });
    if let Some(message) = error
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
    {
        return Err(message);
    }
    let outcomes = outcomes
        .into_inner()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    Ok(failures(programs, outcomes))
}

/// The failures as a report, each with where its example is and what it printed (cut to 15 lines).
pub fn report(failures: &[Failure]) -> String {
    let mut out = String::new();
    for f in failures {
        out.push_str(&format!("{}: example failed\n", f.label));
        let lines: Vec<&str> = f.message.lines().collect();
        for line in lines.iter().take(15) {
            out.push_str(&format!("    {line}\n"));
        }
        if lines.len() > 15 {
            out.push_str(&format!("    ... {} more lines\n", lines.len() - 15));
        }
    }
    out
}
