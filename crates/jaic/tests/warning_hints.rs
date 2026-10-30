//! Warnings carry the fix when it is clear: a format string and its arguments that disagree, code
//! after a `return`, a procedure that can end without one. Each test applies the edit and
//! compiles the result to show the warning is gone and the program still builds.
use jaic::diagnostics::Diagnostic;
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use jaic::source::Applicability;
use std::{path::PathBuf, rc::Rc};

const PRELUDE: &str = "emit :: (format: string, args: ..Any) {} @PrintLike\n";

fn compile(source: &str) -> Vec<Diagnostic> {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    if let Err(e) = compiler.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}\n{source}", e.render(&compiler.src.sources));
    }
    compiler.diag.warnings.clone()
}

fn warning_of(source: &str, message: &str) -> Diagnostic {
    let warnings = compile(source);
    warnings
        .into_iter()
        .find(|w| w.message.starts_with(message))
        .unwrap_or_else(|| panic!("no warning `{message}` in:\n{source}"))
}

fn applied(source: &str, d: &Diagnostic, index: usize) -> String {
    let mut edits: Vec<_> = d.suggestions()[index]
        .replacements()
        .map(|(span, text)| (span.start as usize, span.end as usize, text.to_string()))
        .collect();
    edits.sort_by_key(|e| std::cmp::Reverse(e.0));
    let mut out = source.to_string();
    for (start, end, text) in edits {
        out.replace_range(start..end, &text);
    }
    out
}

const COUNT: &str = "incorrect number of arguments";

/// The program with the call `body` in `main`, after the prelude.
fn program(body: &str) -> String {
    format!("{PRELUDE}main :: () {{\n    n := 1;\n    {body}\n}}\n")
}

/// Apply the first suggestion of the count warning of `body`; the result compiles with no
/// count warning. Returns the new `main` body line.
#[track_caller]
fn fixed_call(body: &str) -> String {
    let source = program(body);
    let d = warning_of(&source, COUNT);
    assert_eq!(
        d.suggestions()[0].applicability,
        Applicability::MaybeIncorrect
    );
    let fixed = applied(&source, &d, 0);
    let again = compile(&fixed);
    assert!(
        again.iter().all(|w| !w.message.starts_with(COUNT)),
        "still warns: {fixed}"
    );
    fixed
        .lines()
        .nth(3)
        .expect("the call line")
        .trim()
        .to_string()
}

#[test]
fn an_extra_argument_is_removed() {
    assert_eq!(fixed_call("emit(\"done\\n\", n, 2);"), "emit(\"done\\n\");");
    assert_eq!(
        fixed_call("emit(\"%\\n\", n, n + 1, 3);"),
        "emit(\"%\\n\", n);"
    );
}

#[test]
fn an_extra_argument_after_a_call_with_brackets_is_removed() {
    assert_eq!(
        fixed_call("emit(\"%\\n\", n, (n + 1) * 2);"),
        "emit(\"%\\n\", n);"
    );
}

#[test]
fn a_percent_without_an_argument_is_escaped() {
    assert_eq!(fixed_call("emit(\"100%\\n\");"), "emit(\"100\\%\\n\");");
    assert_eq!(
        fixed_call("emit(\"% of 100%\\n\", n);"),
        "emit(\"% of 100\\%\\n\", n);"
    );
    assert_eq!(
        fixed_call("emit(\"% and % and %\\n\", n);"),
        "emit(\"% and \\% and \\%\\n\", n);"
    );
}

#[test]
fn an_indexed_directive_gets_no_escape() {
    // `%3` names an argument that is not there: the author meant something.
    let source = program("emit(\"%3\\n\", n);");
    let d = warning_of(&source, COUNT);
    assert!(d.suggestions().is_empty());
}

#[test]
fn a_call_that_does_not_end_at_the_argument_gets_no_removal() {
    // The extra argument is wrapped in parentheses the span leaves out.
    let source = program("emit(\"done\\n\", (n));");
    let d = warning_of(&source, COUNT);
    for (i, _) in d.suggestions().iter().enumerate() {
        let fixed = applied(&source, &d, i);
        assert!(
            compile(&fixed)
                .iter()
                .all(|w| !w.message.starts_with(COUNT)),
            "{fixed}"
        );
    }
}

#[test]
fn code_after_a_return_can_be_removed() {
    let source = "main :: () {\n    return;\n    x := 1;\n    y := 2;\n}\n";
    let d = warning_of(source, "statements in block after `return`");
    assert_eq!(
        d.suggestions()[0].applicability,
        Applicability::MaybeIncorrect
    );
    let fixed = applied(source, &d, 0);
    assert_eq!(fixed, "main :: () {\n    return;\n    \n}\n");
    assert!(compile(&fixed).is_empty());
}

#[test]
fn a_procedure_that_can_end_without_returning_points_at_its_end() {
    let source =
        "f :: (n: int) -> int {\n    if n > 0 return 1;\n}\nmain :: () {\n    x := f(1);\n}\n";
    let d = warning_of(source, "not all control paths return a value");
    assert_eq!(d.notes.len(), 1);
    let (span, text) = &d.notes[0];
    assert_eq!(text, "the body ends here without a `return`");
    assert_eq!(&source[span.start as usize..span.end as usize], "}");
    assert_eq!(span.start as usize, source.find("}\nmain").unwrap());
}
