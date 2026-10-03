use jai_ir::{ProcedureId, Program};
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-procedure-notes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        for (path, text) in files {
            let path = directory.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(directory)
    }
    fn program(&self) -> Program {
        let graph = ModuleGraph::load(
            &self.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
        .unwrap();
        resolve_graph(&graph).unwrap_or_else(|error| panic!("{}", error.render(graph.sources())))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn named(program: &Program, name: &str) -> Vec<ProcedureId> {
    program
        .library()
        .debug_sources()
        .unwrap()
        .procedures()
        .filter_map(|(id, source)| (source.name == name).then_some(id))
        .collect()
}
fn returned(program: &Program) -> i128 {
    match jai_vm::execute(program, Limits::default()).outcome {
        Outcome::Complete(values) => match values.as_slice() {
            [Value::Int(value)] => value.value(),
            other => panic!("integer expected: {other:?}"),
        },
        other => panic!("source should complete: {other:?}"),
    }
}

#[test]
fn imported_procedure_notes_keep_the_defining_file_through_callable_aliases() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Tools::#import \"Tools\"; alias::Tools.answer; main::()->int{return alias();}",
        ),
        (
            "modules/Tools.jai",
            "answer::()->int #no_debug {return 42;} @PrintLike @Reason(\"original\", context) host::() #foreign; @Host",
        ),
    ]);
    let program = fixture.program();
    assert_eq!(returned(&program), 42);
    let answers = named(&program, "answer");
    let [answer] = answers.as_slice() else {
        panic!("one canonical source definition");
    };
    let notes = program
        .library()
        .debug_sources()
        .unwrap()
        .procedure_notes(*answer);
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].text(), b"PrintLike");
    assert_eq!(notes[1].text(), b"Reason(\"original\", context)");
    assert_eq!(
        notes[0].location().path(),
        fixture.0.join("modules/Tools.jai").canonicalize().unwrap()
    );
    assert!(
        !program
            .library()
            .debug_sources()
            .unwrap()
            .procedure_policy(*answer)
            .emits()
    );
    let hosts = named(&program, "host");
    let [host] = hosts.as_slice() else {
        panic!("one prototype source");
    };
    assert_eq!(
        program
            .library()
            .debug_sources()
            .unwrap()
            .procedure_notes(*host)[0]
            .text(),
        b"Host"
    );
}

#[test]
fn generic_and_nested_procedures_keep_notes_under_distinct_runtime_identities() {
    let fixture = Fixture::new(&[(
        "main.jai",
        r#"
        identity::(value:$T)->T{return value;} @Generic
        main::()->int {
            nested::()->int #no_debug {return 21;} @Nested
            half:=nested();
            left:s32=identity(cast(s32)half);
            right:s64=identity(cast(s64)21);
            return left+right;
        }
    "#,
    )]);
    let program = fixture.program();
    assert_eq!(returned(&program), 42);
    let generics = named(&program, "identity");
    assert_eq!(generics.len(), 2);
    assert_ne!(generics[0], generics[1]);
    for id in generics {
        assert_eq!(
            program
                .library()
                .debug_sources()
                .unwrap()
                .procedure_notes(id)[0]
                .text(),
            b"Generic"
        );
    }
    let nested_procedures = named(&program, "nested");
    let [nested] = nested_procedures.as_slice() else {
        panic!("one nested procedure");
    };
    assert_eq!(
        program
            .library()
            .debug_sources()
            .unwrap()
            .procedure_notes(*nested)[0]
            .text(),
        b"Nested"
    );
}

#[test]
fn arbitrary_note_arguments_are_source_metadata_and_do_not_execute() {
    let fixture = Fixture::new(&[(
        "main.jai",
        r#"
        seen:int=0;
        bump::()->int {seen+=1;return seen;}
        main::()->int {return 42+seen;} @Reason(bump(), unknown_name, "bytes\x00")
    "#,
    )]);
    let program = fixture.program();
    assert_eq!(returned(&program), 42);
    let main_procedures = named(&program, "main");
    let [main] = main_procedures.as_slice() else {
        panic!("one entry procedure");
    };
    assert_eq!(
        program
            .library()
            .debug_sources()
            .unwrap()
            .procedure_notes(*main)[0]
            .text(),
        b"Reason(bump(), unknown_name, \"bytes\\x00\")"
    );
}

#[test]
fn legacy_resolution_without_a_source_snapshot_reports_missing_note_provenance() {
    let module = jai_syntax::parse("main::()->int{return 42;} @SourceFact").unwrap();
    let error = jai_sema::resolve(&module).unwrap_err();
    assert_eq!(
        error.message,
        "procedure notes require their defining source snapshot"
    );
}

#[test]
fn caller_scope_insertion_preserves_the_imported_procedure_note_source() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Quoted::#import \"Quoted\"; main::()->int{CALLER::42; #insert,scope() Quoted.Quote;}",
        ),
        (
            "modules/Quoted.jai",
            "CALLER::99; Quote::#code{nested::()->int #no_debug {return CALLER;} @FromQuote return nested();};",
        ),
    ]);
    let program = fixture.program();
    assert_eq!(returned(&program), 42);
    let nested = named(&program, "nested");
    let [nested] = nested.as_slice() else {
        panic!("one inserted procedure");
    };
    let notes = program
        .library()
        .debug_sources()
        .unwrap()
        .procedure_notes(*nested);
    assert_eq!(notes[0].text(), b"FromQuote");
    assert_eq!(
        notes[0].location().path(),
        fixture.0.join("modules/Quoted.jai").canonicalize().unwrap()
    );
}
