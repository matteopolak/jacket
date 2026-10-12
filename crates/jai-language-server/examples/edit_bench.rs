//! How long jailsp takes to answer after an edit inside a procedure body.
//!
//! `cargo run --release -p jai-language-server --example edit_bench -- [--edits N] [--stdlib DIR] DIR FILE LINE`
//!
//! DIR is opened as a workspace folder and FILE (under it) as the one open document, as an
//! editor would. Each round inserts a statement before 1-based LINE (or removes the one inserted
//! last time), then asks for diagnostics and hovers the first identifier of the line. Prints
//! each round's time and how many compiles it started. See docs/tools/edit-bench.md.
use jai_language_server::{DocumentUri, Environment, Limits, Position, Range, Session, TextChange};
use std::path::PathBuf;
use std::time::Instant;

fn environment(stdlib: PathBuf) -> Environment {
    Environment {
        fs: std::rc::Rc::new(jaic::sema::NativeFs),
        options: Box::new(move |_| {
            let mut options = jaic::sema::Options::host();
            options.import_paths = vec![stdlib.clone()];
            options.preload = Some(stdlib.join("prelude/module.jai"));
            options
        }),
    }
}

fn main() {
    let mut edits: i32 = 6;
    let mut stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut rest = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--edits" => edits = args.next().and_then(|n| n.parse().ok()).expect("--edits N"),
            "--stdlib" => stdlib = PathBuf::from(args.next().expect("--stdlib DIR")),
            _ => rest.push(arg),
        }
    }
    let [dir, file, line] = rest.as_slice() else {
        panic!("usage: edit_bench [--edits N] [--stdlib DIR] DIR FILE LINE");
    };
    let dir = std::fs::canonicalize(dir).expect("a directory");
    let file = std::fs::canonicalize(file).expect("a file");
    let line: u32 = line.parse::<u32>().expect("a line number") - 1;
    let text = std::fs::read_to_string(&file).expect("a readable file");
    let target = text.lines().nth(line as usize).expect("LINE inside FILE");
    let indent = target.len() - target.trim_start().len();
    let statement = format!("{}bench_edit := 1;\n", &target[..indent]);
    let uri = DocumentUri::parse(&format!("file://{}", file.display())).expect("a file URI");

    let mut session = Session::with_environment(Limits::default(), environment(stdlib));
    session.set_workspace_folders(vec![dir]);
    let started = Instant::now();
    session.open(uri.clone(), 1, text).expect("open");
    let answer = |session: &Session, inserted: bool| {
        let line = line + inserted as u32;
        let diagnostics = session.diagnostics(&uri).expect("diagnostics").len();
        let hover = session
            .hover(
                &uri,
                Position {
                    line,
                    character: indent as u32,
                },
            )
            .expect("hover")
            .is_some();
        (diagnostics, hover)
    };
    let (diagnostics, hover) = answer(&session, false);
    println!(
        "open: {:.0} ms, {} compiles, {diagnostics} diagnostics, hover {hover}",
        started.elapsed().as_secs_f64() * 1e3,
        session.compile_counts().0
    );
    let mut total = 0.0;
    for round in 0..edits {
        let inserted = round % 2 == 0;
        let at = Position {
            line,
            character: 0,
        };
        let range = if inserted {
            Range {
                start: at,
                end: at,
            }
        } else {
            Range {
                start: at,
                end: Position {
                    line: line + 1,
                    character: 0,
                },
            }
        };
        let change = TextChange {
            range: Some(range),
            range_length: None,
            text: if inserted {
                statement.clone()
            } else {
                String::new()
            },
        };
        let before = session.compile_counts().0;
        let started = Instant::now();
        session.change(&uri, round + 2, &[change]).expect("change");
        let (diagnostics, hover) = answer(&session, inserted);
        let ms = started.elapsed().as_secs_f64() * 1e3;
        total += ms;
        println!(
            "edit {round}: {ms:.0} ms, {} compiles, {diagnostics} diagnostics, hover {hover}",
            session.compile_counts().0 - before
        );
    }
    // Every compile is over: the profile (`JAIC_PROFILE=1`) covers them all.
    drop(session);
    if let Some(report) = jaic::interp::profile::report(30) {
        eprintln!("{report}");
    }
    if edits > 0 {
        println!("mean edit: {:.0} ms", total / edits as f64);
    }
}
