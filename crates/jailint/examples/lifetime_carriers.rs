//! Inventory semantic contracts by checked procedure instance and target.
//!
//! `cargo run -p jailint --example lifetime_carriers -- root.jai --os=windows --cpu=x64`
//! Records include unchecked templates and bodyless boundaries explicitly; those records
//! are not evidence that a carrier's body was checked or its contract was inferred.
#![forbid(unsafe_code)]
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProcId, ProgramSource, procs::BodyState};
use std::{path::PathBuf, rc::Rc};

const NOTES: &[&str] = &[
    "FreesArgument",
    "GrowsArray",
    "AddsToArray",
    "RemovesFromArray",
    "RemovesElement",
    "MovesElements",
    "ResetsElements",
    "BorrowsElement",
    "Locks",
    "Unlocks",
    "Opens",
    "Closes",
    "AppendsString",
    "FormatsToString",
    "TemporaryStorageMark",
    "ResetsTemporaryStorage",
    "RestoresTemporaryStorage",
    "TempAllocates",
    "PrintLike",
    "Interpolates",
    "Cold",
];

fn main() {
    let mut os = None;
    let mut cpu = None;
    let mut roots = Vec::new();
    for arg in std::env::args().skip(1) {
        if let Some(name) = arg.strip_prefix("--os=") {
            os = Some(jaic::target::parse_os(name).expect("supported --os target"));
        } else if let Some(name) = arg.strip_prefix("--cpu=") {
            cpu = Some(jaic::target::parse_cpu(name).expect("supported --cpu target"));
        } else {
            roots.push(PathBuf::from(arg));
        }
    }
    assert!(!roots.is_empty(), "supply one or more program roots");
    let stdlib = jaic::default_stdlib_dir();
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    jaic::target::select_os_cpu(&mut options, os, cpu);
    let target = format!(
        "{}-{}",
        jaic::target::os_name(options.os),
        jaic::target::cpu_name(options.cpu)
    );
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual, IdeRoot::under(&stdlib)]);
    facts.lint = true;
    compiler.ed.record(facts);
    let sources: Vec<_> = roots.iter().cloned().map(ProgramSource::File).collect();
    compiler.begin_sources(&sources).unwrap();
    compiler.finish_program().unwrap();
    let diagnostics = compiler.ide_check_all();
    assert!(
        diagnostics.is_empty(),
        "incomplete inventory: {diagnostics:?}"
    );
    let facts = compiler.ed.facts().unwrap();
    let mut checked = 0;
    let mut carriers = 0;
    for (index, proc) in compiler.pr.procs.iter().enumerate() {
        let id = ProcId(index as u32);
        let declarations: Vec<_> = proc
            .lit
            .header
            .notes
            .iter()
            .chain(compiler.pr.proc_decl_notes.get(&id).into_iter().flatten())
            .collect();
        let notes: Vec<_> = declarations.iter().filter_map(|n| note_family(n)).collect();
        if notes.is_empty() {
            continue;
        }
        let contracts: Vec<_> = declarations.iter().filter(|n| note_family(n).is_some())
            .map(|n| serde_json::json!({"text": n.text.as_ref(), "start": n.span.start, "end": n.span.end})).collect();
        let compiler_read: Vec<_> = ["PrintLike", "Interpolates", "Cold"]
            .into_iter()
            .filter(|name| compiler.proc_has_note(id, name))
            .collect();
        carriers += 1;
        let valid = proc.body_state == BodyState::Done
            && facts
                .lifetime_bodies
                .get(&id)
                .is_some_and(|body| std::sync::Arc::ptr_eq(&proc.lit, &body.syntax));
        checked += usize::from(valid);
        println!(
            "{}",
            serde_json::json!({
                "target": target, "procedure": index, "name": proc.name.as_str(),
                "path": compiler.src.sources.get(proc.span.file).path.to_string(),
                "start": proc.span.start, "end": proc.span.end, "notes": notes,
            "contracts": contracts, "compiler_read": compiler_read,
                "checked": valid, "template": proc.is_poly, "bodyless": proc.lit.body.is_none(),
            })
        );
    }
    assert!(
        carriers > 0 && checked > 0,
        "empty carrier coverage is not a proof"
    );
    eprintln!(
        "lifetime-carriers target={target} carriers={carriers} checked={checked} roots={roots:?}"
    );
}

fn note_family(note: &jaic::ast::Note) -> Option<&str> {
    let name = note.text.split('(').next()?.trim();
    NOTES.contains(&name).then_some(name)
}
