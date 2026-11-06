//! Exercise the real Node runner with an owned deterministic engine, without building Wasm.
use super::*;
use test_support::TempDir;

/// A counter proves engine reuse; `slow` exercises a synchronous call the supervisor must stop.
/// For example, two successive programs `0`, `1` require one retained engine instance.
fn runtime() -> Option<TempDir> {
    match Command::new("node").arg("--version").output() {
        Ok(result) => assert!(result.status.success(), "Node version probe failed"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("skipping optional browser-example runner test: install Node to run it");
            return None;
        }
        Err(error) => panic!("could not probe Node: {error}"),
    }
    let dir = TempDir::new("doc-example-node-runtime");
    std::fs::write(dir.join("jai_wasm.wasm"), b"owned fixture").unwrap();
    std::fs::write(
        dir.join("engine.mjs"),
        r"export async function createEngine() {
  let calls = 0;
  return { play(files) {
    const text = files['main.jai'];
    if (text === 'slow') {
      const end = Date.now() + 1500;
      while (Date.now() < end) {}
      return {exitCode:0, diagnostics:[]};
    }
    if (text === 'fail-after-timeout') return {exitCode:1, stderr:'follow-up was checked'};
    if (text === 'worker-exit') process.exit(23);
    if (text === 'compiler-crash') {
      calls++;
      const error = new Error('owned compiler crash');
      error.compilerCrashed = true;
      throw error;
    }
    const actual = calls++;
    return {exitCode:Number(text) === actual ? 0 : 1, stderr:`expected ${text}, observed ${actual}`};
  }};
}
",
    )
    .unwrap();
    Some(dir)
}

fn program(label: &str, text: &str, timeout: u64) -> Program {
    Program {
        label: label.into(),
        text: text.into(),
        timeout: Some(timeout),
    }
}

#[test]
fn browser_validation_reuses_each_shards_engine_between_successful_examples() {
    let Some(runtime) = runtime() else {
        return;
    };
    let shards = threads();
    let programs: Vec<_> = (0..shards * 2)
        .map(|i| Program {
            label: "counter".into(),
            text: (i / shards).to_string(),
            // Large budgets must not wrap Node's timer into an immediate one-millisecond expiry.
            timeout: (i < shards).then_some(u64::MAX),
        })
        .collect();
    let failures = run(
        &Engine::Browser {
            node: "node".into(),
            runtime: &runtime,
        },
        &programs,
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "engine state was reset between successful examples"
    );
}

#[test]
fn browser_validation_enforces_each_fence_timeout_and_continues_the_shard() {
    let Some(runtime) = runtime() else {
        return;
    };
    let mut programs = vec![program("slow", "slow", 1)];
    programs.extend((1..threads()).map(|_| program("fast", "0", 5)));
    programs.push(program("after", "fail-after-timeout", 5));
    let failures = run(
        &Engine::Browser {
            node: "node".into(),
            runtime: &runtime,
        },
        &programs,
    )
    .unwrap();
    let labels: Vec<_> = failures
        .iter()
        .map(|failure| failure.label.as_str())
        .collect();
    assert_eq!(labels, ["slow", "after"]);
    assert!(failures[0].message.contains("Stopped after 1"));
    assert_eq!(failures[1].message, "follow-up was checked");
}

#[test]
fn browser_validation_reaps_an_exited_worker_and_continues_the_shard() {
    let Some(runtime) = runtime() else {
        return;
    };
    let mut programs = vec![program("exit", "worker-exit", 5)];
    programs.extend((1..threads()).map(|_| program("fast", "0", 5)));
    programs.push(program("after", "fail-after-timeout", 5));
    let failures = run(
        &Engine::Browser {
            node: "node".into(),
            runtime: &runtime,
        },
        &programs,
    )
    .unwrap();
    let labels: Vec<_> = failures
        .iter()
        .map(|failure| failure.label.as_str())
        .collect();
    assert_eq!(labels, ["exit", "after"]);
    assert!(failures[0].message.contains("status 23"));
    assert_eq!(failures[1].message, "follow-up was checked");
}

#[test]
fn browser_validation_resets_a_crashed_engine_without_resetting_other_shards() {
    let Some(runtime) = runtime() else {
        return;
    };
    let shards = threads();
    let programs: Vec<_> = (0..shards * 3)
        .map(|i| {
            let round = i / shards;
            let text = if i == shards {
                "compiler-crash".into()
            } else if i == shards * 2 {
                "0".into()
            } else {
                round.to_string()
            };
            program(
                if i == shards {
                    "crash"
                } else {
                    "counter"
                },
                &text,
                5,
            )
        })
        .collect();
    let failures = run(
        &Engine::Browser {
            node: "node".into(),
            runtime: &runtime,
        },
        &programs,
    )
    .unwrap();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].label, "crash");
    assert_eq!(failures[0].message, "owned compiler crash");
}

#[test]
fn browser_validation_uses_the_supplied_runtime_for_every_library_root() {
    use clap::Parser;
    use std::ffi::OsString;

    let Some(runtime) = runtime() else {
        return;
    };
    let cli = crate::cli::Cli::try_parse_from([
        OsString::from("jacket"),
        OsString::from("doc"),
        OsString::from("--validate-examples"),
        OsString::from("--runtime"),
        runtime.as_os_str().to_owned(),
    ])
    .unwrap();
    let crate::cli::Command::Doc(args) = cli.command else {
        panic!("doc command")
    };
    let site = super::super::site::Site::new(
        Vec::new(),
        Vec::new(),
        "/".into(),
        String::new(),
        String::new(),
        String::new(),
        true,
    );
    site.add_runnable(program("provided runtime", "0", 5));
    let stdlib = runtime.join("stdlib");
    for root in [
        stdlib.clone(),
        stdlib.join("selected"),
        runtime.join("authored"),
    ] {
        std::fs::create_dir_all(&root).unwrap();
        assert!(
            super::super::test_examples(
                &args,
                &site,
                &runtime.join("unavailable-interpreter"),
                &stdlib,
                &root,
            )
            .unwrap(),
            "supplied runtime was bypassed for {}",
            root.display(),
        );
    }
    site.add_runnable(program("bad source", "fail-after-timeout", 5));
    assert!(
        !super::super::test_examples(
            &args,
            &site,
            &runtime.join("unavailable-interpreter"),
            &stdlib,
            &stdlib,
        )
        .unwrap(),
        "a failing supplied runtime must not be reported as successful",
    );
}
