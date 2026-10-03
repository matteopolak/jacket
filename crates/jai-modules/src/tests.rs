use std::fs;

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-modules-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(root)
    }
    fn graph(&self) -> Result<ModuleGraph, GraphError> {
        ModuleGraph::load(
            &self.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn lookup(
    graph: &ModuleGraph,
    file: FileInstanceId,
    names: &[&str],
) -> Result<Binding, LookupError> {
    let symbols: Vec<_> = names
        .iter()
        .map(|name| graph.symbols().find(name).unwrap())
        .collect();
    graph.lookup(
        file,
        &NamePath {
            root: symbols[0],
            members: symbols[1..].to_vec(),
        },
    )
}
fn declaration(graph: &ModuleGraph, binding: Binding) -> &Declaration {
    let Binding::Declaration(id) = binding else {
        panic!("expected declaration")
    };
    graph.declaration(id).unwrap()
}
fn root_file(graph: &ModuleGraph) -> FileInstanceId {
    graph.module(graph.root()).unwrap().entry()
}
#[test]
fn local_procedure_overloads_keep_distinct_declaration_identities() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "pick :: (x: int) -> int { return x; } pick :: (x: bool) -> bool { return x; } main :: () -> int { return pick(3); }",
    )]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(set) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected overload set");
    };
    let members = graph.overload_set(set).unwrap().declarations();
    assert_eq!(members.len(), 2);
    assert_ne!(members[0], members[1]);
    assert!(members.iter().all(|id| matches!(
        graph.declaration(*id).unwrap().syntax().kind,
        FileDeclarationKind::Procedure(_)
    )));
}

#[test]
fn module_private_overloads_do_not_leak_into_exported_membership() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Lib :: #import \"Lib\"; main :: () -> int { return Lib.pick(3); }",
        ),
        (
            "modules/Lib.jai",
            "pick :: (x: int) -> int { return x; } #scope_module pick :: (x: bool) -> bool { return x; } #scope_export pick :: (x: u8) -> u8 { return x; }",
        ),
    ]);
    let graph = fixture.graph().unwrap();
    let Binding::Module(module) = lookup(&graph, root_file(&graph), &["Lib"]).unwrap() else {
        panic!("expected module");
    };
    let file = graph.module(module).unwrap().entry();
    let Binding::OverloadSet(local) = lookup(&graph, file, &["pick"]).unwrap() else {
        panic!("expected local overload set");
    };
    let Binding::OverloadSet(exported) =
        lookup(&graph, root_file(&graph), &["Lib", "pick"]).unwrap()
    else {
        panic!("expected exported overload set");
    };
    assert_eq!(graph.overload_set(local).unwrap().declarations().len(), 3);
    assert_eq!(
        graph.overload_set(exported).unwrap().declarations().len(),
        2
    );
    assert_ne!(local, exported);
}

#[test]
fn imported_procedures_merge_with_application_overloads_without_copying_declarations() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#import \"Lib\"; pick :: (x: bool) -> bool { return x; } main :: () {}",
        ),
        ("modules/Lib.jai", "pick :: (x: int) -> int { return x; }"),
    ]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected imported overload group");
    };
    let members = graph.overload_set(group).unwrap().declarations();
    assert_eq!(members.len(), 2);
    let owners = members
        .iter()
        .map(|id| {
            graph
                .file(graph.declaration(*id).unwrap().file())
                .unwrap()
                .module()
        })
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(owners.len(), 2);
}

#[test]
fn imported_overload_groups_deduplicate_and_keep_private_members_separate() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#import \"Outer\"; #import \"Inner\"; Outer::#import \"Outer\"; main::(){}",
        ),
        (
            "modules/Outer.jai",
            "#import \"Inner\"; pick::(x:bool){} #scope_module pick::(x:u8){}",
        ),
        (
            "modules/Inner.jai",
            "pick::(x:int){} #scope_module pick::(x:u16){}",
        ),
    ]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected reexported overloads");
    };
    assert_eq!(graph.overload_set(group).unwrap().declarations().len(), 2);
    assert_eq!(
        lookup(&graph, root_file(&graph), &["Outer", "pick"]).unwrap(),
        Binding::OverloadSet(group)
    );
    let Binding::Module(outer) = lookup(&graph, root_file(&graph), &["Outer"]).unwrap() else {
        panic!("expected namespace");
    };
    let Binding::OverloadSet(internal) =
        lookup(&graph, graph.module(outer).unwrap().entry(), &["pick"]).unwrap()
    else {
        panic!("expected internal overloads");
    };
    assert_eq!(
        graph.overload_set(internal).unwrap().declarations().len(),
        3
    );
}

#[test]
fn imported_callable_merge_does_not_relax_value_or_type_collisions() {
    for declaration in ["pick::3;", "pick::struct{value:int;}", "pick:int;"] {
        let fixture = Fixture::new(&[
            (
                "main.jai",
                &format!("#import \"Lib\"; {declaration} main::(){{}}"),
            ),
            ("modules/Lib.jai", "pick::(x:int)->int{return x;}"),
        ]);
        assert!(
            fixture
                .graph()
                .unwrap_err()
                .to_string()
                .contains("conflicting declaration")
        );
    }
}

#[test]
fn modern_basic_math_logging_shape_retains_both_original_prototypes() {
    let fixture = Fixture::new(&[
        ("main.jai", "#import \"Basic\";main::(){}"),
        (
            "modules/Basic.jai",
            "#import \"Math\";log::(format:string,args:..Any)->s64 #foreign;",
        ),
        (
            "modules/Math.jai",
            "log::(value:float64)->float64 #foreign;",
        ),
    ]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["log"]).unwrap() else {
        panic!("expected logging and numerical prototypes");
    };
    let members = graph.overload_set(group).unwrap().declarations();
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|id| matches!(
        graph.declaration(*id).unwrap().syntax().kind,
        FileDeclarationKind::ProcedurePrototype(_)
    )));
}

#[test]
fn overload_sets_reexport_the_same_declarations() {
    let fixture = Fixture::new(&[
        ("main.jai", "Outer :: #import \"Outer\"; main :: () {}"),
        ("modules/Outer.jai", "#import \"Inner\";"),
        (
            "modules/Inner.jai",
            "pick :: (x: int) -> int { return x; } pick :: (x: bool) -> bool { return x; }",
        ),
    ]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(outer) =
        lookup(&graph, root_file(&graph), &["Outer", "pick"]).unwrap()
    else {
        panic!("expected overload set");
    };
    let originating_declaration = graph.overload_set(outer).unwrap().declarations()[0];
    let inner_module = graph
        .file(graph.declaration(originating_declaration).unwrap().file())
        .unwrap()
        .module();
    let inner = graph.module(inner_module).unwrap();
    let name = graph.symbols().find("pick").unwrap();
    assert_eq!(
        inner.exports().get(&name),
        Some(&Binding::OverloadSet(outer))
    );
    assert_eq!(graph.overload_set(outer).unwrap().declarations().len(), 2);
}
#[test]
fn modules_are_isolated_and_qualification_filters_exports() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "app_only :: 90; Lib :: #import \"Lib\"; main :: () -> int { return Lib.answer(); }",
        ),
        (
            "modules/Lib.jai",
            "answer :: () -> int { return hidden; } #scope_module hidden :: 42;",
        ),
    ]);
    let graph = f.graph().unwrap();
    let app = root_file(&graph);
    let Binding::Module(lib) = lookup(&graph, app, &["Lib"]).unwrap() else {
        panic!()
    };
    let lib_file = graph.module(lib).unwrap().entry();
    assert!(matches!(
        lookup(&graph, lib_file, &["app_only"]),
        Err(LookupError::UnknownName(_))
    ));
    assert!(lookup(&graph, lib_file, &["hidden"]).is_ok());
    assert!(matches!(
        lookup(&graph, app, &["Lib", "hidden"]),
        Err(LookupError::PrivateMember { .. })
    ));
    assert_eq!(
        declaration(&graph, lookup(&graph, app, &["Lib", "answer"]).unwrap()).file(),
        lib_file
    );
    assert_eq!(graph.modules().len(), 2);
}
#[test]
fn loads_create_independent_file_scopes_and_share_module_exports() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#scope_file private :: 1; #load \"a.jai\"; #load \"b.jai\"; #load \"./a.jai\"; main :: () {}",
        ),
        ("a.jai", "exported_a :: 10; #scope_file private :: 2;"),
        ("b.jai", "exported_b :: 20; #scope_file private :: 3;"),
    ]);
    let graph = f.graph().unwrap();
    assert_eq!(graph.files().len(), 3);
    assert_eq!(graph.loads().len(), 3);
    assert_eq!(graph.loads()[0].target(), graph.loads()[2].target());
    let files = graph.module(graph.root()).unwrap().files();
    let bindings: Vec<_> = files
        .iter()
        .map(|&file| lookup(&graph, file, &["private"]).unwrap())
        .collect();
    assert_ne!(bindings[0], bindings[1]);
    assert_ne!(bindings[1], bindings[2]);
    for &file in files {
        assert!(lookup(&graph, file, &["exported_a"]).is_ok());
        assert!(lookup(&graph, file, &["exported_b"]).is_ok());
    }
    assert!(
        !graph
            .module(graph.root())
            .unwrap()
            .exports()
            .contains_key(&graph.symbols().find("private").unwrap())
    );
}
#[test]
fn anonymous_alias_using_and_reexports_preserve_declaration_identity() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#import \"Wrapper\"; First :: #import \"Leaf\"; Second :: #import \"Leaf\"; using Also :: #import \"Leaf\"; main :: () {}",
        ),
        (
            "modules/Wrapper/module.jai",
            "#import \"Leaf\"; Alias :: #import \"Leaf\";",
        ),
        (
            "modules/Leaf.jai",
            "value :: 42; #scope_module hidden :: 9;",
        ),
    ]);
    let graph = f.graph().unwrap();
    let file = root_file(&graph);
    let value = lookup(&graph, file, &["value"]).unwrap();
    assert_eq!(value, lookup(&graph, file, &["First", "value"]).unwrap());
    assert_eq!(value, lookup(&graph, file, &["Second", "value"]).unwrap());
    assert_eq!(value, lookup(&graph, file, &["Alias", "value"]).unwrap());
    assert_eq!(
        lookup(&graph, file, &["First"]).unwrap(),
        lookup(&graph, file, &["Second"]).unwrap()
    );
    assert_eq!(graph.modules().len(), 3);
    assert!(matches!(
        lookup(&graph, file, &["hidden"]),
        Err(LookupError::UnknownName(_))
    ));
}
#[test]
fn file_private_import_does_not_leak_and_same_source_has_separate_instances() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#scope_file #import \"Leaf\"; #load \"other.jai\"; Local :: #import,file \"shared.jai\"; #load \"shared.jai\"; main :: () {}",
        ),
        ("other.jai", "other :: 3;"),
        ("shared.jai", "shared :: 2;"),
        ("modules/Leaf.jai", "value :: 42;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    let other = graph
        .files()
        .iter()
        .find(|file| {
            graph
                .sources()
                .get(file.source())
                .unwrap()
                .path()
                .file_name()
                .unwrap()
                == "other.jai"
        })
        .unwrap();
    assert!(lookup(&graph, root, &["value"]).is_ok());
    assert!(matches!(
        lookup(&graph, other.id(), &["value"]),
        Err(LookupError::UnknownName(_))
    ));
    let app = declaration(&graph, lookup(&graph, root, &["shared"]).unwrap());
    let imported = declaration(&graph, lookup(&graph, root, &["Local", "shared"]).unwrap());
    assert_ne!(app.id(), imported.id());
    assert_ne!(app.file(), imported.file());
    assert_eq!(
        graph.file(app.file()).unwrap().source(),
        graph.file(imported.file()).unwrap().source()
    );
}
#[test]
fn file_directory_imports_and_exported_loads() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "One :: #import,file \"local/entry.jai\"; Two :: #import,dir \"local\"; main :: () {}",
        ),
        ("local/entry.jai", "value :: 1;"),
        ("local/module.jai", "#scope_file #load \"loaded.jai\";"),
        ("local/loaded.jai", "value :: 2;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    let one = declaration(&graph, lookup(&graph, root, &["One", "value"]).unwrap());
    let two = declaration(&graph, lookup(&graph, root, &["Two", "value"]).unwrap());
    assert_ne!(one.id(), two.id());
    assert_eq!(
        graph
            .sources()
            .get(two.location().source)
            .unwrap()
            .path()
            .file_name()
            .unwrap(),
        "loaded.jai"
    );
}
#[test]
fn collisions_cycles_and_unsupported_forms_fail_explicitly() {
    let collision = Fixture::new(&[
        ("main.jai", "value :: 1;\n#import \"Leaf\";"),
        ("modules/Leaf.jai", "value :: 2;"),
    ])
    .graph()
    .unwrap_err();
    assert!(
        matches!(&collision, GraphError::Located { diagnostic, .. } if diagnostic.location.span.start > 0)
    );
    assert!(collision.to_string().contains("main.jai:2:"));
    for (files, kind) in [
        (
            vec![("main.jai", "#load \"main.jai\";")],
            DependencyKind::Load,
        ),
        (
            vec![
                ("main.jai", "#import \"A\";"),
                ("modules/A.jai", "#import \"B\";"),
                ("modules/B.jai", "#import \"A\";"),
            ],
            DependencyKind::Import,
        ),
    ] {
        let error = Fixture::new(&files).graph().unwrap_err();
        assert!(error.to_string().contains(":1:"), "{error}");
        assert!(
            matches!(error, GraphError::Cycle { kind: actual, location: Some(_), .. } if actual == kind)
        );
    }
    for source in [
        "#import \"Leaf\"()(DEBUG=true);",
        "#import,string \"value :: 1;\";",
        "#if true { #load \"other.jai\"; }",
    ] {
        assert!(
            Fixture::new(&[("main.jai", source), ("modules/Leaf.jai", "value :: 1;")])
                .graph()
                .is_err(),
            "{source}"
        );
    }
}
#[test]
fn imported_file_parse_errors_keep_original_source_path() {
    let error = Fixture::new(&[
        ("main.jai", "#import \"Bad\";"),
        ("modules/Bad.jai", "\nbad :: () -> int { return +; }"),
    ])
    .graph()
    .unwrap_err();
    assert!(matches!(error, GraphError::Located { .. }));
    assert!(error.to_string().contains("Bad.jai:2:"), "{error}");
}

fn parameter(graph: &ModuleGraph, file: FileInstanceId, name: &str) -> ParameterValue {
    let Binding::Parameter(id) = lookup(graph, file, &[name]).unwrap() else {
        panic!("expected parameter")
    };
    graph.parameter(id).unwrap().value.clone()
}
#[test]
fn parameter_instances_preserve_ordered_request_identity_and_private_typed_values() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "BASE :: 3; A :: #import \"Lib\"(Count=BASE + 1, Flag=true); B :: #import \"Lib\"(Count=4, Flag=true); C :: #import \"Lib\"(Flag=true, Count=4); D :: #import \"Lib\"; E :: #import \"Lib\"(); F :: #import \"Lib\"(Count=4);",
        ),
        (
            "modules/Lib.jai",
            "#module_parameters(Count: u8 = 4, Flag := false, Label: string = \"x\"); state: int = 0; #load \"extra.jai\";",
        ),
        ("modules/extra.jai", "value :: Count + 1;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    let ids: Vec<_> = ["A", "B", "C", "D", "E", "F"]
        .iter()
        .map(|n| {
            let Binding::Module(id) = lookup(&graph, root, &[n]).unwrap() else {
                panic!()
            };
            id
        })
        .collect();
    assert_eq!(ids[0], ids[1]);
    for i in 2..ids.len() {
        assert!(ids[..i].iter().all(|id| *id != ids[i]));
    }
    let first = graph.module(ids[0]).unwrap().entry();
    assert_eq!(
        parameter(&graph, first, "Count"),
        ParameterValue::Scalar(jai_eval::Value::Int(
            jai_eval::Integer::checked(jai_eval::IntegerType::U8, 4).unwrap()
        ))
    );
    assert_eq!(
        parameter(&graph, first, "Label"),
        ParameterValue::String("x".into())
    );
    assert!(matches!(
        lookup(&graph, root, &["A", "Count"]),
        Err(LookupError::PrivateMember { .. })
    ));
    assert_ne!(
        lookup(&graph, root, &["A", "state"]),
        lookup(&graph, root, &["C", "state"])
    );
    assert_eq!(graph.module(ids[0]).unwrap().files().len(), 2);
}
#[test]
fn program_parameters_share_values_across_instances_and_enforce_first_import() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"Lib\"()(Debug=true); B :: #import \"Lib\"(Count=9); W :: #import \"Wrapper\";",
        ),
        (
            "modules/Lib.jai",
            "#module_parameters(Count := 4)(Debug := false); value :: Debug;",
        ),
        ("modules/Wrapper.jai", "Other :: #import \"Lib\"(Count=2);"),
    ]);
    let graph = f.graph().unwrap();
    let debug = graph.symbols().find("Debug").unwrap();
    assert_eq!(
        graph
            .parameters()
            .iter()
            .filter(|p| p.name == debug)
            .count(),
        3
    );
    assert!(
        graph
            .parameters()
            .iter()
            .filter(|p| p.name == debug)
            .all(|p| p.program_wide
                && p.value == ParameterValue::Scalar(jai_eval::Value::Bool(true)))
    );
    for source in [
        "A :: #import \"Lib\"; B :: #import \"Lib\"()(Debug=true);",
        "A :: #import \"Lib\"()(Debug=true); B :: #import \"Lib\"()(Debug=true);",
    ] {
        let error = Fixture::new(&[
            ("main.jai", source),
            ("modules/Lib.jai", "#module_parameters()(Debug := false);"),
        ])
        .graph()
        .unwrap_err();
        assert!(
            error.to_string().contains("may be supplied once"),
            "{error}"
        );
    }
}
#[test]
fn parameter_blocks_and_forward_loaded_constants_select_real_dependencies() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "#if FLAG { #load \"yes.jai\"; } else { #load \"missing.jai\"; } A :: #import \"Lib\"(N=COUNT); #load \"settings.jai\";",
        ),
        ("settings.jai", "FLAG :: true; COUNT :: 5;"),
        (
            "yes.jai",
            "#scope_file private :: 9; #scope_export selected :: 42;",
        ),
        (
            "modules/Lib.jai",
            "#module_parameters(N := DEFAULT) { DEFAULT :: 2; } #if N == 5 { answer :: 5; #load \"five.jai\"; } else { answer :: 2; }",
        ),
        ("modules/five.jai", "extra :: N;"),
    ]);
    let graph = f.graph().unwrap();
    let root = root_file(&graph);
    assert!(lookup(&graph, root, &["selected"]).is_ok());
    assert!(matches!(
        lookup(&graph, root, &["private"]),
        Err(LookupError::UnknownName(_))
    ));
    assert!(lookup(&graph, root, &["A", "answer"]).is_ok());
    assert!(lookup(&graph, root, &["A", "extra"]).is_ok());
    assert!(matches!(
        lookup(&graph, root, &["A", "DEFAULT"]),
        Err(LookupError::PrivateMember { .. })
    ));
    assert_eq!(graph.loads().len(), 3);
}
#[test]
fn parameter_failures_are_typed_and_source_located() {
    for arguments in [
        "(Required=1, Flag=2)",
        "(Required=256)",
        "(Required=true)",
        "(Unknown=1)",
        "(Required=1, Required=2)",
        "(Required=1, 2)",
        "()",
    ] {
        let source = format!("Lib :: #import \"Lib\"{arguments};");
        let error = Fixture::new(&[
            ("main.jai", &source),
            (
                "modules/Lib.jai",
                "#module_parameters(Required: u8, Flag := false);",
            ),
        ])
        .graph()
        .unwrap_err();
        assert!(matches!(error, GraphError::Located { .. }), "{error}");
        assert!(error.to_string().contains(".jai:1:"), "{error}");
    }
    let error = Fixture::new(&[
        ("main.jai", "#load \"other.jai\";"),
        ("other.jai", "#module_parameters(X := true);"),
    ])
    .graph()
    .unwrap_err();
    assert!(error.to_string().contains("module entry file"));
    let error = Fixture::new(&[("main.jai", "#if UNKNOWN { #load \"missing.jai\"; }")])
        .graph()
        .unwrap_err();
    assert!(matches!(error, GraphError::Pending { .. }));
}
#[test]
fn suspended_imports_keep_pending_diagnostics_and_program_request_order() {
    let error = Fixture::new(&[
        ("main.jai", "A :: #import \"A\"; main :: () {}"),
        ("modules/A.jai", "#if missing { value :: 1; }"),
    ])
    .graph()
    .unwrap_err();
    assert!(matches!(error, GraphError::Pending { .. }), "{error}");
    assert!(error.to_string().contains("A.jai:1:"), "{error}");
    let graph = Fixture::new(&[
        (
            "main.jai",
            "First :: #import \"A\"()(DEBUG=LATER); #load \"values.jai\"; Second :: #import \"A\";",
        ),
        ("values.jai", "LATER :: true;"),
        ("modules/A.jai", "#module_parameters()(DEBUG := false);"),
    ])
    .graph()
    .unwrap();
    assert!(
        graph
            .parameters()
            .iter()
            .all(|p| p.value == ParameterValue::Scalar(jai_eval::Value::Bool(true)))
    );
}
#[test]
fn parameter_defaults_schedule_forward_references_and_type_defaults_bind() {
    let graph = Fixture::new(&[
        ("main.jai", "Lib :: #import \"Lib\";"),
        ("modules/Lib.jai", "#module_parameters(A := B + 1, B := 3); #if A == 4 { chosen :: 1; } else #if false { discarded :: 1; } else { other :: 1; }"),
    ]).graph().unwrap();
    assert!(lookup(&graph, root_file(&graph), &["Lib", "chosen"]).is_ok());
    assert!(lookup(&graph, root_file(&graph), &["Lib", "other"]).is_err());
    let typed = Fixture::new(&[
        ("main.jai", "Lib :: #import \"Lib\";"),
        (
            "modules/Lib.jai",
            "#module_parameters(T: Type = int) { X :: 4; }",
        ),
    ])
    .graph()
    .unwrap();
    assert!(
        typed
            .parameters()
            .iter()
            .any(|p| matches!(p.value, ParameterValue::Type(_)))
    );
    let error = Fixture::new(&[(
        "main.jai",
        "#if f() { X :: 1; } f :: () -> bool { return true; }",
    )])
    .graph()
    .unwrap_err();
    assert!(matches!(error, GraphError::Pending { .. }), "{error}");
}
#[test]
fn float_module_parameters_preserve_precision_identity_and_float_conditions() {
    let graph = Fixture::new(&[
        ("main.jai", "A :: #import \"Float\"(Amount=0h00000000); B :: #import \"Float\"(Amount=0h80000000); C :: #import \"Float\"(Amount=0h00000000); W :: #import \"Wide\"(Amount=1.25 + 0.5);"),
        ("modules/Float.jai", "#module_parameters(Amount := 0.0); #if Amount { nonzero :: 1; } else { zero :: 1; }"),
        ("modules/Wide.jai", "#module_parameters(Amount: float64 = cast(float64) 0.0); #if Amount == 1.75 { answer :: 42; }"),
    ]).graph().unwrap();
    let root = root_file(&graph);
    assert_eq!(lookup(&graph, root, &["A"]), lookup(&graph, root, &["C"]));
    assert_ne!(lookup(&graph, root, &["A"]), lookup(&graph, root, &["B"]));
    assert!(lookup(&graph, root, &["A", "zero"]).is_ok());
    assert!(lookup(&graph, root, &["B", "zero"]).is_ok());
    assert!(lookup(&graph, root, &["W", "answer"]).is_ok());
    let Binding::Module(wide) = lookup(&graph, root, &["W"]).unwrap() else {
        panic!()
    };
    assert_eq!(
        parameter(&graph, graph.module(wide).unwrap().entry(), "Amount"),
        ParameterValue::Scalar(jai_eval::Value::Float(jai_types::FloatValue::from_f64(
            1.75
        )))
    );
    let error = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"Float\"(Amount=cast(float64) 1.0);",
        ),
        ("modules/Float.jai", "#module_parameters(Amount := 0.0);"),
    ])
    .graph()
    .unwrap_err();
    assert!(
        error.to_string().contains("cannot narrow float precision"),
        "{error}"
    );
}

#[test]
fn callable_aliases_join_groups_with_forward_original_targets() {
    let f = Fixture::new(&[(
        "main.jai",
        "pick :: (x: int) -> int { return x; } pick :: alias; alias :: target; target :: (x: bool) -> bool { return x; } main :: () {}",
    )]);
    let graph = f.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected callable group")
    };
    let members = graph.overload_set(group).unwrap().declarations();
    assert_eq!(members.len(), 2);
    assert!(members.iter().all(|id| matches!(
        graph.declaration(*id).unwrap().syntax().kind,
        FileDeclarationKind::Procedure(_)
    )));
    let Binding::Declaration(target) = lookup(&graph, root_file(&graph), &["target"]).unwrap()
    else {
        panic!("expected target")
    };
    assert!(members.contains(&target));
    assert_eq!(
        graph
            .declarations()
            .iter()
            .filter(|d| matches!(d.syntax().kind, FileDeclarationKind::Constant(_)))
            .count(),
        2
    );
}

#[test]
fn callable_aliases_preserve_exported_and_private_membership() {
    let f = Fixture::new(&[
        ("main.jai", "Lib :: #import \"Lib\"; main :: () {}"),
        (
            "modules/Lib.jai",
            "pick :: (x: int) -> int { return x; } #scope_module hidden :: (x: bool) -> bool { return x; } pick :: hidden; #scope_export pick :: byte_target; byte_target :: (x: u8) -> u8 { return x; }",
        ),
    ]);
    let graph = f.graph().unwrap();
    let Binding::Module(module) = lookup(&graph, root_file(&graph), &["Lib"]).unwrap() else {
        panic!("expected module")
    };
    let file = graph.module(module).unwrap().entry();
    let Binding::OverloadSet(local) = lookup(&graph, file, &["pick"]).unwrap() else {
        panic!("expected local group")
    };
    let Binding::OverloadSet(exported) =
        lookup(&graph, root_file(&graph), &["Lib", "pick"]).unwrap()
    else {
        panic!("expected exported group")
    };
    assert_eq!(graph.overload_set(local).unwrap().declarations().len(), 3);
    assert_eq!(
        graph.overload_set(exported).unwrap().declarations().len(),
        2
    );
}

#[test]
fn callable_aliases_allow_explicit_qualified_foreign_module_targets() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "Lib :: #import \"Lib\"; pick :: (x: int) -> int { return x; } pick :: Lib.pick; main :: () {}",
        ),
        ("modules/Lib.jai", "pick :: (x: bool) -> bool { return x; }"),
    ]);
    let graph = f.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected group")
    };
    let Binding::Declaration(target) = lookup(&graph, root_file(&graph), &["Lib", "pick"]).unwrap()
    else {
        panic!("expected original imported target")
    };
    assert!(
        graph
            .overload_set(group)
            .unwrap()
            .declarations()
            .contains(&target)
    );
}

#[test]
fn callable_alias_collisions_reject_scalar_and_type_targets() {
    for target in ["target :: 42;", "target :: struct { value: int; }"] {
        let source = format!(
            "pick :: (x: int) -> int {{ return x; }} pick :: target; {target} main :: () {{}}"
        );
        let f = Fixture::new(&[("main.jai", &source)]);
        let error = f.graph().unwrap_err();
        assert!(
            error
                .to_string()
                .contains("overload alias does not denote a procedure"),
            "{error}"
        );
    }
}

#[test]
fn callable_alias_collisions_reject_cycles_at_original_alias_source() {
    let f = Fixture::new(&[(
        "main.jai",
        "pick :: (x: int) -> int { return x; } pick :: first; first :: second; second :: first; main :: () {}",
    )]);
    let error = f.graph().unwrap_err();
    assert!(
        error
            .to_string()
            .contains("cyclic procedure overload alias"),
        "{error}"
    );
}

#[test]
fn callable_alias_collisions_preserve_qualified_privacy() {
    let f = Fixture::new(&[
        (
            "main.jai",
            "Lib :: #import \"Lib\"; pick :: (x: int) -> int { return x; } pick :: Lib.hidden; main :: () {}",
        ),
        (
            "modules/Lib.jai",
            "#scope_module hidden :: (x: bool) -> bool { return x; }",
        ),
    ]);
    let error = f.graph().unwrap_err();
    assert!(
        error.to_string().contains("target member is private"),
        "{error}"
    );
}

#[test]
fn callable_alias_resolution_uses_a_bounded_iterative_source_walk() {
    let mut source = String::from("pick :: (x: int) -> int { return x; } pick :: alias_0; ");
    for index in 0..6000 {
        source.push_str(&format!("alias_{index} :: alias_{}; ", index + 1));
    }
    source
        .push_str("alias_6000 :: target; target :: (x: bool) -> bool { return x; } main :: () {}");
    let f = Fixture::new(&[("main.jai", &source)]);
    let graph = f.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected callable group")
    };
    assert_eq!(graph.overload_set(group).unwrap().declarations().len(), 2);
}

#[test]
fn callable_alias_groups_merge_multiple_forward_candidates_in_source_order() {
    let f = Fixture::new(&[(
        "main.jai",
        "pick :: first; pick :: (x: int) -> int { return x; } pick :: second; first :: (x: bool) -> bool { return x; } second :: (x: u8) -> u8 { return x; } main :: () {}",
    )]);
    let graph = f.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected callable group")
    };
    assert_eq!(graph.overload_set(group).unwrap().declarations().len(), 3);
}

#[test]
fn callable_alias_guard_waits_for_a_pure_conditional_load() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "pick::(x:int)->bool{return false;}pick::alias;#if true{#load \"later.jai\";}#if #run pick(true){answer::42;}else{answer::0;}",
        ),
        ("later.jai", "alias::(x:bool)->bool{return true;}"),
    ]);
    let mut discovery = GraphDiscovery::new(
        &fixture.0.join("main.jai"),
        GraphOptions::default(),
        &Filesystem,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.pending_conditions().count(), 1);
    let graph = discovery.graph();
    let Binding::OverloadSet(group) = lookup(graph, root_file(graph), &["pick"]).unwrap() else {
        panic!("guard snapshot needs the complete overload family");
    };
    assert_eq!(graph.overload_set(group).unwrap().declarations().len(), 2);
    assert_eq!(graph.files().len(), 2);
}

#[test]
fn callable_alias_guard_waits_for_a_later_namespace_import() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "pick::(x:int)->bool{return false;}pick::Library.target;#if #run pick(true){answer::42;}else{answer::0;}Library::#import \"Library\";",
        ),
        (
            "modules/Library.jai",
            "target::(x:bool)->bool{return true;}",
        ),
    ]);
    let mut discovery = GraphDiscovery::new(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![fixture.0.join("modules")],
        },
        &Filesystem,
    )
    .unwrap();
    assert!(!discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.pending_conditions().count(), 1);
    let graph = discovery.graph();
    let Binding::OverloadSet(group) = lookup(graph, root_file(graph), &["pick"]).unwrap() else {
        panic!("guard snapshot needs the imported target");
    };
    let members = graph.overload_set(group).unwrap().declarations();
    assert_eq!(members.len(), 2);
    assert_ne!(
        graph.declaration(members[0]).unwrap().file(),
        graph.declaration(members[1]).unwrap().file()
    );
}

#[test]
fn callable_aliases_include_pending_target_groups_without_export_leaks() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "pick::(x:u8)->u8{return x;}pick::family;family::(x:int)->int{return x;}family::later;later::(x:bool)->bool{return x;}",
    )]);
    let graph = fixture.graph().unwrap();
    let Binding::OverloadSet(group) = lookup(&graph, root_file(&graph), &["pick"]).unwrap() else {
        panic!("expected complete target family");
    };
    assert_eq!(graph.overload_set(group).unwrap().declarations().len(), 3);
}
