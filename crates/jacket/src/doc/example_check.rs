//! Testing the examples of a site: every one with a Run button is run before the site is written,
//! the way the browser runs it, so a Run button never opens on an error.
use crate::test::exec::{Invocation, run_bounded};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

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
    let outcomes = crate::parallel::map(programs, crate::parallel::threads(), |i, program| {
        interpret(jaic, stdlib, import_dir, program, dir, i)
    });
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

    if let Err(e) = std::fs::write(&file, &program.text) {
        return Some(format!("cannot write {}: {e}", file.display()));
    }

    let mut args = vec![file.to_string_lossy().into_owned()];

    args.extend(["-interp", "-quiet", "-os", "wasm"].map(String::from));
    args.extend(
        import_dir
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned()),
    );
    let invocation = Invocation {
        program: jaic.to_path_buf(),
        args,
        cwd: dir.to_path_buf(),
        env: vec![(
            jacket_config::env::STDLIB.name.to_string(),
            stdlib.to_string_lossy().into_owned(),
        )],
        pipe_input: false,
    };
    let limit = program.timeout.map_or(DEFAULT_TIMEOUT, Duration::from_secs);
    let ran = run_bounded(&invocation, Some(limit));
    let _ = std::fs::remove_file(&file);
    let ran = match ran {
        Ok(ran) => ran,
        Err(e) => return Some(format!("could not run `{}`: {e}", jaic.display())),
    };
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&ran.stdout),
        String::from_utf8_lossy(&ran.stderr)
    );

    match ran.status {
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
            // A string, so a limit past what a double holds is still exact.
            let seconds = p.timeout.unwrap_or(DEFAULT_TIMEOUT.as_secs());
            serde_json::json!({ "text": p.text, "timeoutSeconds": seconds.to_string() })
        })
        .collect();

    std::fs::write(&list, serde_json::to_vec(&json).map_err(|e| e.to_string())?)
        .map_err(|e| format!("cannot write {}: {e}", list.display()))?;
    let shards = crate::parallel::threads().min(programs.len().max(1));
    let indices: Vec<usize> = (0..shards).collect();
    let found = crate::parallel::try_map(&indices, shards, |_, &shard| {
        let output = Command::new(node)
            .arg(&script)
            .arg(runtime)
            .arg(&list)
            .arg(shard.to_string())
            .arg(shards.to_string())
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("could not start `{}`: {e}", node.display()))?;
        let mut seen = 0;
        let mut failed = Vec::new();

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
                failed.push((i, value["message"].as_str().unwrap_or("").to_owned()));
            }
        }

        let expected = (shard..programs.len()).step_by(shards).count();

        if seen < expected {
            return Err(format!(
                "the browser compiler stopped after {seen} of {expected} examples:\n{}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }

        Ok(failed)
    })?;
    let mut outcomes = vec![None; programs.len()];

    for (i, message) in found.into_iter().flatten() {
        outcomes[i] = Some(message);
    }

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

/// The error for `--runtime` with no `node` to run the examples.
pub fn node_missing() -> String {
    format!(
        "{}\nnote: or drop --runtime to test the examples in jaic's interpreter",
        crate::prerequisite::missing(
            &jaic_toolchain::package::Need::Node,
            "node",
            "jacket doc --runtime"
        )
    )
}

/// `--runtime` runs the examples under Node after the site is built: find out before any of that
/// work that `node` is there.
pub fn require_node(args: &super::DocArgs) -> Result<(), String> {
    if args.runtime.is_some() && !args.check && !args.no_test_examples {
        crate::prerequisite::require(
            &jaic_toolchain::package::Need::Node,
            "node",
            "jacket doc --runtime",
        )
        .map_err(|_| node_missing())?;
    }

    Ok(())
}
