//! Selected operator publications keep original declaration and lexical scope.
use jai_modules::{GraphDiscovery, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(source: &str, library: &str) -> Result<jai_ir::Program, String> {
    compile_files(&[
        ("/operator-using/main.jai", source),
        ("/operator-using/library.jai", library),
    ])
}

fn compile_files(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    let mut overlay = SourceOverlay::new();
    for &(path, text) in files {
        overlay
            .insert(Path::new(path), text.as_bytes().to_vec())
            .unwrap();
    }
    let mut discovery = GraphDiscovery::new(Path::new(files[0].0), Default::default(), &overlay)
        .map_err(|error| error.to_string())?;
    for _ in 0..16 {
        if discovery
            .advance()
            .map_err(|error| error.to_string())?
            .is_complete()
        {
            break;
        }
        let requests = discovery.pending_using_requests();
        if requests.is_empty() {
            return Err("operator using discovery suspended without a using request".into());
        }
        let outcome = jai_sema::resolve_discovery_using(
            discovery.graph(),
            &requests,
            &Default::default(),
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())?;
        if outcome.decisions.is_empty() && !outcome.pending.is_empty() {
            return Err(format!(
                "operator using remains pending: {:?}",
                outcome.pending
            ));
        }
        for (id, decision) in outcome.decisions {
            discovery
                .resolve_using(id, decision)
                .map_err(|error| error.to_string())?;
        }
    }
    let graph = discovery
        .into_graph()
        .map_err(|_| "operator using discovery did not reach completion".to_owned())?;
    jai_sema::resolve_graph(&graph).map_err(|error| error.to_string())
}

fn run(source: &str, library: &str) -> i128 {
    let program = compile(source, library).unwrap();
    run_program(&program)
}

fn run_program(program: &jai_ir::Program) -> i128 {
    let execution = jai_vm::execute(program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("operator using did not complete: {execution:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result: {values:?}");
    };
    value.value()
}

const LIBRARY: &str = r#"
Box::struct{value:int;}
operator +::(a:Box,b:Box)->Box{return .{value=a.value+b.value};}
operator *::(a:Box,b:Box)->Box{return .{value=a.value*b.value};}
"#;

#[test]
fn aliases_resolve_after_checked_using_publishes_selected_original_operators() {
    let bridge = r#"
Library::#import,file "library.jai";
using,only(.["+"]) Library;
"#;
    let source = r#"
Library::#import,file "library.jai";
Bridge::#import,file "bridge.jai";
operator+::Bridge.operator+;
main::()->int{
    a:Library.Box=.{value=3}; b:Library.Box=.{value=4};
    aliased:=a+b;
    using,only(.["+"]) Bridge;
    selected:=a+b;
    return aliased.value+selected.value;
}
"#;
    let program = compile_files(&[
        ("/operator-using/main.jai", source),
        ("/operator-using/bridge.jai", bridge),
        ("/operator-using/library.jai", LIBRARY),
    ])
    .unwrap();
    assert_eq!(run_program(&program), 14);
    let rejected = source.replace(":=a+b", ":=a*b");
    let error = compile_files(&[
        ("/operator-using/main.jai", &rejected),
        ("/operator-using/bridge.jai", bridge),
        ("/operator-using/library.jai", LIBRARY),
    ])
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("expected integer operands"),
        "{error}"
    );
}

#[test]
fn lexical_only_operator_filter_publishes_exact_selected_ids() {
    assert_eq!(
        run(
            r#"
Lib::#import,file "library.jai";
main::()->int{
    a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
    using,only(.["+"]) Lib;
    return (a+b).value;
}
"#,
            LIBRARY
        ),
        7
    );
    let error = compile(
        r#"
Lib::#import,file "library.jai";
main::()->int{
    a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
    using,only(.["+"]) Lib;
    return (a*b).value;
}
"#,
        LIBRARY,
    )
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("expected integer operands"),
        "{error}"
    );
}

#[test]
fn lexical_except_operator_filter_does_not_expose_excluded_group() {
    assert_eq!(
        run(
            r#"
Lib::#import,file "library.jai";
main::()->int{
    a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
    using,except(.["+"]) Lib;
    return (a*b).value;
}
"#,
            LIBRARY
        ),
        12
    );
    let error = compile(
        r#"
Lib::#import,file "library.jai";
main::()->int{
    a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
    using,except(.["+"]) Lib;
    return (a+b).value;
}
"#,
        LIBRARY,
    )
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("expected integer operands"),
        "{error}"
    );
}

#[test]
fn selected_unary_operator_shadows_outer_binary_token_group() {
    let error = compile(
        r#"
Lib::#import,file "library.jai";
operator +::(a:Lib.Box,b:Lib.Box)->Lib.Box{return .{value=99};}
main::()->int{
    a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
    using,only(.["+"]) Lib;
    return (a+b).value;
}
"#,
        "Box::struct{value:int;} operator +::(a:Box)->Box{return a;}",
    )
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("expected integer operands"),
        "{error}"
    );
}

#[test]
fn earlier_local_callable_does_not_capture_later_using_operators() {
    let error = compile(
        r#"
Lib::#import,file "library.jai";
main::()->int{
    calculate::()->int{
        a:Lib.Box=.{value=3}; b:Lib.Box=.{value=4};
        return (a+b).value;
    }
    using,only(.["+"]) Lib;
    return calculate();
}
"#,
        LIBRARY,
    )
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("expected integer operands"),
        "{error}"
    );
}
