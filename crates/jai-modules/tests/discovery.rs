use jai_modules::{
    Binding, ConditionSelectionError, DiscoveryConditionContext, DiscoveryStatus, GraphDiscovery,
    GraphError, GraphOptions, LookupError, ModuleBoundArgument, ParameterValue, SourceOverlay,
    SourceProvider, SourceSpecializationKey,
};
use jai_syntax::{ExpressionKind, FileDeclarationKind, NamePath, StatementKind};
use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

#[derive(Default)]
struct Inputs {
    source: SourceOverlay,
    reads: RefCell<HashMap<PathBuf, usize>>,
}
impl Inputs {
    fn new(files: &[(&str, &str)]) -> Self {
        let mut inputs = Self::default();
        for (path, source) in files {
            inputs
                .source
                .insert(Path::new(path), source.as_bytes().to_vec())
                .unwrap();
        }
        inputs
    }
    fn reads(&self, path: &str) -> usize {
        self.reads
            .borrow()
            .get(Path::new(path))
            .copied()
            .unwrap_or(0)
    }
}
impl SourceProvider for Inputs {
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        self.source.canonicalize(path)
    }
    fn read(&self, path: &Path) -> io::Result<Vec<u8>> {
        *self.reads.borrow_mut().entry(path.to_owned()).or_default() += 1;
        self.source.read(path)
    }
    fn is_file(&self, path: &Path) -> bool {
        self.source.is_file(path)
    }
}

#[test]
fn lexical_conditions_retain_original_context_and_resume_canonical_private_modules() {
    let source = "enabled :: true; main :: () { enabled :: #run false; #if enabled { Missing :: #import,file \"absent.jai\"; } else { First :: #import,file \"dependency.jai\"; Second :: #import,file \"./dependency.jai\"; } }";
    let inputs = Inputs::new(&[
        ("/discovery/main.jai", source),
        (
            "/discovery/dependency.jai",
            "value :: 7; #scope_module; hidden :: 9;",
        ),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    let DiscoveryStatus::Awaiting {
        conditions,
        dependencies,
    } = discovery.advance().unwrap()
    else {
        panic!("expected source condition request")
    };
    assert_eq!(conditions.len(), 1);
    assert!(dependencies.is_empty());
    let request = discovery.condition(conditions[0]).unwrap();
    let condition_location = request.location;
    assert_eq!(request.location.span.text(source), "enabled");
    assert!(matches!(request.expression.kind, ExpressionKind::Name(_)));
    let DiscoveryConditionContext::Lexical {
        declaration,
        scopes,
    } = &request.context
    else {
        panic!("expected lexical context")
    };
    assert!(matches!(
        discovery
            .graph()
            .declaration(*declaration)
            .unwrap()
            .syntax()
            .kind,
        FileDeclarationKind::Procedure(_)
    ));
    assert!(scopes.iter().flat_map(|scope| &scope.statements).any(|statement| matches!(&statement.kind, StatementKind::Constant(constant) if matches!(constant.initializer.kind, ExpressionKind::CompileTime(_)))));
    let original = discovery
        .graph()
        .declarations()
        .iter()
        .map(|declaration| (declaration.id(), declaration.location()))
        .collect::<Vec<_>>();
    let original_source = discovery.graph().files()[0].source();
    assert!(matches!(
        discovery.advance().unwrap(),
        DiscoveryStatus::Awaiting { .. }
    ));
    assert_eq!(discovery.conditions().len(), 1);
    assert_eq!(
        discovery
            .graph()
            .declarations()
            .iter()
            .map(|declaration| (declaration.id(), declaration.location()))
            .collect::<Vec<_>>(),
        original
    );
    discovery.select_condition(conditions[0], false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.into_graph().ok().unwrap();
    assert_eq!(
        graph.selected_condition(graph.files()[0].id(), condition_location.span),
        Some(false)
    );
    assert_eq!(
        graph.source_condition_selections()[0].location(),
        condition_location
    );
    assert_eq!(graph.files()[0].source(), original_source);
    assert_eq!(
        &graph
            .declarations()
            .iter()
            .map(|declaration| (declaration.id(), declaration.location()))
            .collect::<Vec<_>>()[..original.len()],
        original
    );
    assert_eq!(graph.scoped_imports().len(), 2);
    assert_eq!(
        graph.scoped_imports()[0].module(),
        graph.scoped_imports()[1].module()
    );
    let module = graph.scoped_imports()[0].module();
    assert!(matches!(
        graph.lookup_module(module, &[graph.symbols().find("hidden").unwrap()]),
        Err(LookupError::PrivateMember { .. })
    ));
    let file = graph.files()[0].id();
    assert!(matches!(
        graph.lookup(
            file,
            &NamePath {
                root: graph.symbols().find("First").unwrap(),
                members: vec![]
            }
        ),
        Err(LookupError::UnknownName(_))
    ));
    assert_eq!(inputs.reads("/discovery/main.jai"), 1);
    assert_eq!(inputs.reads("/discovery/dependency.jai"), 1);
    assert_eq!(inputs.reads("/discovery/absent.jai"), 0);
}

#[test]
fn nested_file_conditions_publish_selected_declarations_once() {
    let inputs = Inputs::new(&[(
        "/discovery/main.jai",
        "before :: 1; #if #run true { #if #run false { #load \"absent.jai\"; } else { selected :: 7; } } after :: 2;",
    )]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    discovery.advance().unwrap();
    let outer = discovery.pending_conditions().next().unwrap().id;
    assert!(matches!(
        discovery.condition(outer).unwrap().context,
        DiscoveryConditionContext::File
    ));
    discovery.select_condition(outer, true).unwrap();
    discovery.advance().unwrap();
    let inner = discovery.pending_conditions().next().unwrap().id;
    assert_ne!(inner, outer);
    discovery.select_condition(inner, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let selected = discovery.graph().symbols().find("selected").unwrap();
    let graph = discovery.graph();
    let file = graph.module(graph.root()).unwrap().entry();
    let Binding::Declaration(id) = graph
        .lookup(
            file,
            &NamePath {
                root: selected,
                members: vec![],
            },
        )
        .unwrap()
    else {
        panic!("expected selected declaration")
    };
    let count = graph.declarations().len();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().declarations().len(), count);
    assert_eq!(discovery.graph().source_condition_selections().len(), 2);
    assert_eq!(discovery.graph().declaration(id).unwrap().name(), selected);
    assert!(matches!(
        discovery.select_condition(inner, true),
        Err(ConditionSelectionError::AlreadySelected {
            previous: false
        })
    ));
    discovery.select_condition(inner, false).unwrap();
    assert_eq!(inputs.reads("/discovery/main.jai"), 1);
}

#[test]
fn parameter_initialization_resumes_without_republishing_completed_values() {
    let inputs = Inputs::new(&[(
        "/discovery/main.jai",
        "#module_parameters(A := 3, B := late); #if #run false { late :: 9; } else { late :: 4; } #if B == 4 { chosen :: 1; }",
    )]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    discovery.advance().unwrap();
    let graph = discovery.graph();
    assert_eq!(graph.parameters().len(), 1);
    let file = graph.module(graph.root()).unwrap().entry();
    let name = graph.symbols().find("A").unwrap();
    let original = graph
        .lookup(
            file,
            &NamePath {
                root: name,
                members: vec![],
            },
        )
        .unwrap();
    let request = discovery
        .pending_conditions()
        .find(|request| matches!(request.expression.kind, ExpressionKind::CompileTime(_)))
        .unwrap()
        .id;
    discovery.select_condition(request, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    assert_eq!(graph.parameters().len(), 2);
    assert_eq!(
        graph
            .lookup(
                file,
                &NamePath {
                    root: name,
                    members: vec![]
                }
            )
            .unwrap(),
        original
    );
    assert!(
        matches!(&graph.parameters()[0].value, ParameterValue::Scalar(jai_eval::Value::Int(value)) if value.value() == 3)
    );
    let chosen = graph.symbols().find("chosen").unwrap();
    assert!(matches!(
        graph.lookup(
            file,
            &NamePath {
                root: chosen,
                members: vec![]
            }
        ),
        Ok(Binding::Declaration(_))
    ));
    assert!(discovery.pending_conditions().next().is_none());
}

#[test]
fn resumed_same_instance_import_cycles_keep_selected_source_provenance() {
    let inputs = Inputs::new(&[
        (
            "/discovery/main.jai",
            "main :: () { #if #run true { M :: #import,file \"dependency.jai\"; } }",
        ),
        (
            "/discovery/dependency.jai",
            "f :: () { Root :: #import,file \"main.jai\"; }",
        ),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    discovery.advance().unwrap();
    let id = discovery.pending_conditions().next().unwrap().id;
    discovery.select_condition(id, true).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().modules().len(), 2);
    let backedge = discovery
        .graph()
        .scoped_imports()
        .iter()
        .find(|edge| edge.module() == discovery.graph().root())
        .unwrap();
    let location = backedge.location();
    let source = discovery.graph().sources().get(location.source).unwrap();
    assert_eq!(source.path(), Path::new("/discovery/dependency.jai"));
    assert!(location.span.text(source.text()).contains("#import"));
    assert_eq!(inputs.reads("/discovery/main.jai"), 1);
    assert_eq!(inputs.reads("/discovery/dependency.jai"), 1);
    assert!(discovery.advance().unwrap().is_complete());
    assert!(discovery.into_graph().is_ok());
}

#[test]
fn actual_specializations_keep_conditional_import_decisions_independent() {
    let inputs = Inputs::new(&[
        (
            "/discovery/main.jai",
            "choose :: ($enabled: bool) { #if enabled { A :: #import,file \"a.jai\"; } else { B :: #import,file \"b.jai\"; } }",
        ),
        ("/discovery/a.jai", "value :: 1;"),
        ("/discovery/b.jai", "value :: 2;"),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert!(discovery.has_dependency_templates());
    assert!(discovery.pending_conditions().next().is_none());
    assert_eq!(inputs.reads("/discovery/a.jai"), 0);
    assert_eq!(inputs.reads("/discovery/b.jai"), 0);
    let template = discovery.graph().dependency_templates()[0];
    let enabled = discovery.graph().symbols().find("enabled").unwrap();
    let no = SourceSpecializationKey::new(
        template.declaration(),
        template.location(),
        vec![(enabled, ModuleBoundArgument::Boolean(false))],
    );
    let yes = SourceSpecializationKey::new(
        template.declaration(),
        template.location(),
        vec![(enabled, ModuleBoundArgument::Boolean(true))],
    );
    for (key, selected) in [(&no, false), (&yes, true)] {
        assert!(discovery.discover_specialization(key.clone()).unwrap());
        assert!(!discovery.specialization_discovered(key));
        let DiscoveryStatus::Awaiting {
            conditions, ..
        } = discovery.advance().unwrap()
        else {
            panic!("expected specialization condition")
        };
        let request = discovery.condition(conditions[0]).unwrap();
        assert_eq!(request.specialization.as_ref(), Some(key));
        let (file, span) = (request.file, request.location.span);
        discovery.select_condition(conditions[0], selected).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        assert!(discovery.specialization_discovered(key));
        assert!(!discovery.discover_specialization(key.clone()).unwrap());
        assert_eq!(
            discovery
                .graph()
                .selected_semantic_condition_for(file, span, Some(key)),
            Some(selected)
        );
        assert_eq!(discovery.graph().selected_condition(file, span), None);
    }
    assert_eq!(discovery.graph().source_condition_selections().len(), 2);
    assert_eq!(discovery.graph().scoped_imports().len(), 2);
    assert_eq!(inputs.reads("/discovery/main.jai"), 1);
    assert_eq!(inputs.reads("/discovery/a.jai"), 1);
    assert_eq!(inputs.reads("/discovery/b.jai"), 1);
}

#[test]
fn specialized_import_arguments_create_canonical_module_instances_without_ast_substitution() {
    let inputs = Inputs::new(&[
        (
            "/discovery/main.jai",
            "choose :: ($value: int) { M :: #import,file \"dependency.jai\"(X=value); }",
        ),
        ("/discovery/dependency.jai", "#module_parameters(X: int);"),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let template = discovery.graph().dependency_templates()[0];
    let name = discovery.graph().symbols().find("value").unwrap();
    for value in [3, 4] {
        let value = jai_types::Integer::checked(jai_types::IntegerType::S64, value).unwrap();
        let key = SourceSpecializationKey::new(
            template.declaration(),
            template.location(),
            vec![(name, ModuleBoundArgument::Integer(value))],
        );
        discovery.discover_specialization(key.clone()).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        assert!(discovery.specialization_discovered(&key));
    }
    let graph = discovery.graph();
    assert_eq!(graph.scoped_imports().len(), 2);
    assert_ne!(
        graph.scoped_imports()[0].module(),
        graph.scoped_imports()[1].module()
    );
    assert_eq!(graph.modules().len(), 3);
    assert_eq!(graph.parameters().len(), 2);
    assert_eq!(inputs.reads("/discovery/dependency.jai"), 1);
}

#[test]
fn nested_template_activation_keeps_ordinary_prefix_imports_outside_its_key() {
    let inputs = Inputs::new(&[
        (
            "/discovery/main.jai",
            "outer :: () { Prefix :: #import,file \"prefix.jai\"; inner :: ($enabled: bool) { #if enabled { Missing :: #import,file \"absent.jai\"; } else { Selected :: #import,file \"selected.jai\"; } } }",
        ),
        ("/discovery/prefix.jai", "value :: 1;"),
        ("/discovery/selected.jai", "value :: 2;"),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 1);
    assert!(
        discovery.graph().scoped_imports()[0]
            .specialization()
            .is_none()
    );
    let template = discovery.graph().dependency_templates()[0];
    let enabled = discovery.graph().symbols().find("enabled").unwrap();
    let key = SourceSpecializationKey::new(
        template.declaration(),
        template.location(),
        vec![(enabled, ModuleBoundArgument::Boolean(false))],
    );
    discovery.discover_specialization(key.clone()).unwrap();
    discovery.advance().unwrap();
    let request = discovery.pending_conditions().next().unwrap();
    assert_eq!(request.specialization.as_ref(), Some(&key));
    let id = request.id;
    discovery.select_condition(id, false).unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    assert_eq!(discovery.graph().scoped_imports().len(), 2);
    assert!(
        discovery.graph().scoped_imports()[0]
            .specialization()
            .is_none()
    );
    assert_eq!(
        discovery.graph().scoped_imports()[1].specialization(),
        Some(&key)
    );
    assert_eq!(inputs.reads("/discovery/prefix.jai"), 1);
    assert_eq!(inputs.reads("/discovery/absent.jai"), 0);
}

#[test]
fn specialization_arguments_preserve_enum_nominals_and_source_type_bindings() {
    let inputs = Inputs::new(&[
        (
            "/discovery/main.jai",
            "Colors :: enum { RED; BLUE; } choose :: ($color: Colors, $T: Type) { M :: #import,file \"dependency.jai\"(Color=Colors, T=T, value=color); }",
        ),
        (
            "/discovery/dependency.jai",
            "#module_parameters(Color: Type, T: Type, value: Color);",
        ),
    ]);
    let mut discovery = GraphDiscovery::new(
        Path::new("/discovery/main.jai"),
        GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    assert!(discovery.advance().unwrap().is_complete());
    let graph = discovery.graph();
    let template = graph.dependency_templates()[0];
    let color = graph.symbols().find("color").unwrap();
    let ty = graph.symbols().find("T").unwrap();
    let enumeration = graph
        .declarations()
        .iter()
        .find(|declaration| matches!(declaration.syntax().kind, FileDeclarationKind::Enum(_)))
        .unwrap()
        .id();
    for (bits, builtin) in [
        (
            0,
            jai_modules::ModuleBuiltin::Scalar(jai_types::ScalarType::Int(
                jai_types::IntegerType::U8,
            )),
        ),
        (
            1,
            jai_modules::ModuleBuiltin::Scalar(jai_types::ScalarType::Int(
                jai_types::IntegerType::U16,
            )),
        ),
    ] {
        let value = jai_modules::EnumParameter {
            declaration: enumeration,
            value: jai_types::Integer::checked(jai_types::IntegerType::S64, bits).unwrap(),
        };
        let key = SourceSpecializationKey::new(
            template.declaration(),
            template.location(),
            vec![
                (color, ModuleBoundArgument::Enumeration(value)),
                (
                    ty,
                    ModuleBoundArgument::Type(jai_modules::ModuleType::Builtin(builtin)),
                ),
            ],
        );
        discovery.discover_specialization(key.clone()).unwrap();
        assert!(discovery.advance().unwrap().is_complete());
        assert!(discovery.specialization_discovered(&key));
    }
    assert_eq!(discovery.graph().modules().len(), 3);
    assert_eq!(discovery.graph().parameters().len(), 6);
    assert!(discovery.graph().parameters().iter().filter(|parameter| parameter.name == discovery.graph().symbols().find("value").unwrap()).all(|parameter| matches!(parameter.value, ParameterValue::Enumeration(value) if value.declaration == enumeration)));
    assert_eq!(inputs.reads("/discovery/dependency.jai"), 1);
}
