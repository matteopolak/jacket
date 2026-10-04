use jai_ir::*;
use jai_source::{SourceMap, SourceSpan, Span};
use jai_types::{
    CallingConvention, ContextMode, Direction, Integer, IntegerType, ScalarType, TypeRegistry,
    Types, Variadic,
};

struct Fixture {
    types: Types,
    procedure: Procedure,
    debug: DebugSources,
    location: DebugSourceLocation,
}

fn block(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::FallsThrough,
    }
}

fn int(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}

impl Fixture {
    fn new() -> Self {
        let mut types = TypeRegistry::new();
        let boolean = types.scalar(ScalarType::Bool);
        let signature = types
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([boolean]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: Variadic::None,
            })
            .unwrap();
        let id = ProcedureId::new(7);
        let counter = Local::new(id, 0, ScalarType::Int(IntegerType::S64), &types);
        let parameter = Local::new(id, 1, ScalarType::Bool, &types);
        let iterator = counter.integer(&types).unwrap();
        let store = || Statement::StoreInt(iterator.place(), int(1));
        let procedure = Procedure {
            id,
            signature,
            parameters: vec![parameter],
            locals: vec![counter, parameter],
            body: block(vec![
                Statement::Block(block(vec![store()])),
                Statement::If(
                    BoolExpr::Constant(true),
                    block(vec![store()]),
                    block(vec![store()]),
                ),
                Statement::Cases(Cases {
                    default_position: None,
                    default_through: false,
                    subject: Box::new(store()),
                    arms: vec![CaseArm {
                        condition: BoolExpr::Constant(true),
                        body: block(vec![store()]),
                        through: false,
                    }],
                    default: None,
                    flow: Flow::FallsThrough,
                    exhaustive: false,
                }),
                Statement::While {
                    id: LoopId::new(0),
                    condition: LoopCondition::BoundInt(iterator, int(0)),
                    body: block(vec![store()]),
                },
                Statement::Range(RangeLoop {
                    id: LoopId::new(1),
                    iterator,
                    start: int(0),
                    end: int(2),
                    direction: Direction::Forward,
                    body: block(vec![store()]),
                }),
            ]),
            cleanups: vec![block(vec![store()]).into()],
        };
        let mut source_map = SourceMap::default();
        let source_id = source_map.insert(
            "debug-paths.jai".into(),
            "main :: (enabled: bool) { counter := 0; }".into(),
        );
        let source = source_map.get(source_id).unwrap();
        let mut debug = DebugSources::default();
        debug.set_primary_source(source);
        let location = debug
            .source_location(
                source,
                SourceSpan {
                    source: source_id,
                    span: Span::new(0, source.text().len()),
                },
            )
            .unwrap();
        debug.insert(
            id,
            ProcedureSource {
                name: "main".into(),
                location: location.clone(),
            },
        );
        Self {
            types: types.freeze().unwrap(),
            procedure,
            debug,
            location,
        }
    }

    fn root(&self) -> BlockPath {
        BlockPath::procedure(self.procedure.id)
    }

    fn local_source(&self, scope: BlockPath, declaration: LocalDeclaration) -> LocalSource {
        LocalSource {
            name: "counter".into(),
            location: self.location.clone(),
            scope,
            declaration,
        }
    }

    fn finish(self) -> Result<Library, IrError> {
        ProgramBuilder::new(self.types)
            .procedures(vec![self.procedure])
            .debug_sources(self.debug)
            .finish_library()
    }
}

#[test]
fn publication_preserves_real_block_statement_and_local_provenance() {
    let mut fixture = Fixture::new();
    let scope = fixture.root().child(0, DebugBranch::Block);
    let declaration = scope.statement(0);
    let local = fixture.procedure.locals[0].id();
    fixture
        .debug
        .insert_block(scope.clone(), fixture.location.clone());
    fixture
        .debug
        .insert_statement(declaration.clone(), fixture.location.clone());
    fixture.debug.insert_local(
        local,
        fixture.local_source(
            scope.clone(),
            LocalDeclaration::Statement(declaration.clone()),
        ),
    );
    let library = fixture.finish().unwrap();
    let debug = library.debug_sources().unwrap();
    assert_eq!(
        debug.primary_path().unwrap().to_str(),
        Some("debug-paths.jai")
    );
    assert_eq!(debug.block(&scope), debug.statement(&declaration));
    assert_eq!(debug.local(local).unwrap().scope, scope);
}

#[test]
fn semantic_rebinding_removes_obsolete_body_paths_before_republication() {
    let mut fixture = Fixture::new();
    let root = fixture.root();
    let scope = root.child(0, DebugBranch::Block);
    let declaration = scope.statement(0);
    let local = fixture.procedure.locals[0].id();
    fixture
        .debug
        .insert_block(root.clone(), fixture.location.clone());
    fixture
        .debug
        .insert_block(scope.clone(), fixture.location.clone());
    fixture
        .debug
        .insert_statement(declaration.clone(), fixture.location.clone());
    fixture.debug.insert_local(
        local,
        fixture.local_source(scope, LocalDeclaration::Statement(declaration)),
    );
    fixture
        .debug
        .insert_cleanup_parent(fixture.procedure.id, CleanupId::new(0), root);
    fixture
        .debug
        .set_procedure_policy(fixture.procedure.id, DebugPolicy::Suppress);
    // A delayed semantic recheck replaces staging IR before it is immutable.
    fixture.procedure.body.statements.clear();
    fixture.procedure.cleanups.clear();
    fixture.debug.clear_procedure(fixture.procedure.id);
    fixture.debug.insert(
        fixture.procedure.id,
        ProcedureSource {
            name: "main".into(),
            location: fixture.location.clone(),
        },
    );
    let library = fixture.finish().unwrap();
    let sources = library.debug_sources().unwrap();
    assert_eq!(
        sources.primary_path().unwrap().to_str(),
        Some("debug-paths.jai")
    );
    assert_eq!(sources.blocks().count(), 0);
    assert_eq!(sources.statements().count(), 0);
    assert_eq!(sources.locals().count(), 0);
    assert_eq!(
        sources.procedure_policy(ProcedureId::new(7)),
        DebugPolicy::Emit
    );
    assert!(
        sources
            .cleanup_parent(ProcedureId::new(7), CleanupId::new(0))
            .is_none()
    );
}

#[test]
fn publication_rejects_block_path_ending_at_statement() {
    let mut fixture = Fixture::new();
    let path = fixture.root().then([DebugPathStep::Statement(0)]);
    fixture.debug.insert_block(path, fixture.location.clone());
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug block path",
            ..
        })
    ));
}

#[test]
fn publication_rejects_statement_path_ending_at_block() {
    let mut fixture = Fixture::new();
    let path = StatementPath {
        root: fixture.root().root,
        steps: Box::new([]),
    };
    fixture
        .debug
        .insert_statement(path, fixture.location.clone());
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug statement path",
            ..
        })
    ));
}

#[test]
fn publication_rejects_missing_statement_index() {
    let mut fixture = Fixture::new();
    fixture
        .debug
        .insert_statement(fixture.root().statement(99), fixture.location.clone());
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug statement",
            index: 99
        })
    ));
}

#[test]
fn publication_rejects_missing_cleanup_block() {
    let mut fixture = Fixture::new();
    fixture.debug.insert_block(
        BlockPath::cleanup(fixture.procedure.id, CleanupId::new(1)),
        fixture.location.clone(),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug cleanup",
            index: 1
        })
    ));
}

#[test]
fn publication_rejects_branch_inconsistent_with_actual_statement() {
    let mut fixture = Fixture::new();
    fixture.debug.insert_block(
        fixture.root().child(0, DebugBranch::IfThen),
        fixture.location.clone(),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug branch",
            ..
        })
    ));
}

#[test]
fn publication_rejects_missing_case_arm_and_default_blocks() {
    for branch in [DebugBranch::CaseArm(1), DebugBranch::CaseDefault] {
        let mut fixture = Fixture::new();
        fixture
            .debug
            .insert_block(fixture.root().child(2, branch), fixture.location.clone());
        assert!(fixture.finish().is_err());
    }
}

#[test]
fn case_subject_source_path_resolves_a_statement() {
    let mut fixture = Fixture::new();
    let subject = fixture.root().statement(2).child(DebugBranch::CaseSubject);
    fixture
        .debug
        .insert_statement(subject.clone(), fixture.location.clone());
    let library = fixture.finish().unwrap();
    assert!(
        library
            .debug_sources()
            .unwrap()
            .statement(&subject)
            .is_some()
    );
}

#[test]
fn case_subject_cannot_be_published_as_a_block() {
    let mut fixture = Fixture::new();
    fixture.debug.insert_block(
        fixture.root().child(2, DebugBranch::CaseSubject),
        fixture.location.clone(),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug block path",
            ..
        })
    ));
}

#[test]
fn local_scope_must_have_the_exact_local_owner() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let source = fixture.local_source(
        BlockPath::procedure(ProcedureId::new(8)),
        LocalDeclaration::Statement(fixture.root().statement(0)),
    );
    fixture.debug.insert_local(local, source);
    assert!(
        matches!(fixture.finish(), Err(IrError::LocalOwner { expected, actual }) if expected == ProcedureId::new(7) && actual == ProcedureId::new(8))
    );
}

#[test]
fn same_local_slot_from_another_procedure_is_not_an_existing_local() {
    let mut fixture = Fixture::new();
    let foreign = Local::new(
        ProcedureId::new(8),
        fixture.procedure.locals[0].id().index(),
        ScalarType::Int(IntegerType::S64),
        &fixture.types,
    )
    .id();
    let source = fixture.local_source(
        fixture.root(),
        LocalDeclaration::Statement(fixture.root().statement(0)),
    );
    fixture.debug.insert_local(foreign, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug procedure",
            index: 8
        })
    ));
}

#[test]
fn local_id_cannot_reference_nonexistent_slot() {
    let mut fixture = Fixture::new();
    let nonexistent = Local::new(fixture.procedure.id, 99, ScalarType::Bool, &fixture.types).id();
    let source = fixture.local_source(
        fixture.root(),
        LocalDeclaration::Statement(fixture.root().statement(0)),
    );
    fixture.debug.insert_local(nonexistent, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug local",
            index: 99
        })
    ));
}

#[test]
fn parameter_declaration_uses_zero_based_ordinal_not_local_slot() {
    let mut fixture = Fixture::new();
    let parameter = fixture.procedure.parameters[0].id();
    assert_eq!(parameter.index(), 1);
    let source = fixture.local_source(fixture.root(), LocalDeclaration::Parameter(0));
    fixture.debug.insert_local(parameter, source);
    assert!(fixture.finish().is_ok());

    let mut fixture = Fixture::new();
    let parameter = fixture.procedure.parameters[0].id();
    let source = fixture.local_source(fixture.root(), LocalDeclaration::Parameter(1));
    fixture.debug.insert_local(parameter, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug parameter",
            index: 1
        })
    ));
}

#[test]
fn parameter_declaration_must_identify_actual_parameter_local() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let source = fixture.local_source(fixture.root(), LocalDeclaration::Parameter(0));
    fixture.debug.insert_local(local, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug parameter",
            index: 0
        })
    ));
}

#[test]
fn parameter_scope_must_be_procedure_body() {
    let mut fixture = Fixture::new();
    let parameter = fixture.procedure.parameters[0].id();
    let source = fixture.local_source(
        fixture.root().child(0, DebugBranch::Block),
        LocalDeclaration::Parameter(0),
    );
    fixture.debug.insert_local(parameter, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug parameter",
            index: 0
        })
    ));
}

#[test]
fn local_declaration_in_sibling_lexical_scope_is_rejected() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let scope = fixture.root().child(1, DebugBranch::IfElse);
    let declaration = fixture.root().child(1, DebugBranch::IfThen).statement(0);
    let source = fixture.local_source(scope, LocalDeclaration::Statement(declaration));
    fixture.debug.insert_local(local, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug declaration scope",
            index: 0
        })
    ));
}

#[test]
fn controlling_loop_declaration_can_own_child_body_scope() {
    for (index, branch) in [(3, DebugBranch::While), (4, DebugBranch::Range)] {
        let mut fixture = Fixture::new();
        let local = fixture.procedure.locals[0].id();
        let scope = fixture.root().child(index, branch);
        let declaration = fixture.root().statement(index);
        let source = fixture.local_source(
            scope.clone(),
            LocalDeclaration::Statement(declaration.clone()),
        );
        fixture.debug.insert_local(local, source);
        let library = fixture.finish().unwrap();
        let source = library.debug_sources().unwrap().local(local).unwrap();
        assert_eq!(source.scope, scope);
        assert_eq!(source.declaration, LocalDeclaration::Statement(declaration));
    }
}

#[test]
fn cleanup_root_retains_real_procedure_block_as_lexical_parent() {
    let mut fixture = Fixture::new();
    let procedure = fixture.procedure.id;
    let cleanup = CleanupId::new(0);
    let parent = fixture.root().child(0, DebugBranch::Block);
    let cleanup_root = BlockPath::cleanup(procedure, cleanup);
    fixture
        .debug
        .insert_block(cleanup_root.clone(), fixture.location.clone());
    assert!(
        fixture
            .debug
            .insert_cleanup_parent(procedure, cleanup, parent.clone())
            .is_none()
    );
    let library = fixture.finish().unwrap();
    let debug = library.debug_sources().unwrap();
    assert_eq!(debug.cleanup_parent(procedure, cleanup), Some(&parent));
    assert!(debug.block(&cleanup_root).is_some());
    assert!(debug.cleanup_parent(procedure, CleanupId::new(1)).is_none());
}

#[test]
fn cleanup_lexical_parent_must_have_the_exact_procedure_owner() {
    let mut fixture = Fixture::new();
    let foreign_owner = ProcedureId::new(8);
    fixture.debug.insert_cleanup_parent(
        fixture.procedure.id,
        CleanupId::new(0),
        BlockPath::procedure(foreign_owner),
    );
    // Retain the foreign identity as a real prototype so the owner mismatch,
    // rather than a missing identity, is what publication checks here.
    let prototype = ProcedurePrototype {
        id: foreign_owner,
        signature: fixture.procedure.signature,
        origin: PrototypeOrigin::Compiler,
    };
    let result = ProgramBuilder::new(fixture.types)
        .procedures(vec![fixture.procedure])
        .prototypes(vec![prototype])
        .debug_sources(fixture.debug)
        .finish_library();
    assert!(
        matches!(result, Err(IrError::LocalOwner { expected, actual })
        if expected == ProcedureId::new(7) && actual == foreign_owner)
    );
}

#[test]
fn cleanup_lexical_parent_entry_requires_an_actual_cleanup() {
    let mut fixture = Fixture::new();
    fixture
        .debug
        .insert_cleanup_parent(fixture.procedure.id, CleanupId::new(99), fixture.root());
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug cleanup",
            index: 99
        })
    ));
}

#[test]
fn cleanup_lexical_parent_requires_an_actual_block_path() {
    for steps in [
        vec![DebugPathStep::Statement(0)],
        vec![
            DebugPathStep::Statement(0),
            DebugPathStep::Child(DebugBranch::IfThen),
        ],
        vec![
            DebugPathStep::Statement(99),
            DebugPathStep::Child(DebugBranch::Block),
        ],
    ] {
        let mut fixture = Fixture::new();
        fixture.debug.insert_cleanup_parent(
            fixture.procedure.id,
            CleanupId::new(0),
            fixture.root().then(steps),
        );
        assert!(fixture.finish().is_err());
    }
}

#[test]
fn cleanup_lexical_parent_cannot_reference_missing_cleanup_root() {
    let mut fixture = Fixture::new();
    fixture.debug.insert_cleanup_parent(
        fixture.procedure.id,
        CleanupId::new(0),
        BlockPath::cleanup(fixture.procedure.id, CleanupId::new(99)),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug cleanup",
            index: 99
        })
    ));
}

#[test]
fn nested_cleanup_root_lexical_parent_chain_is_valid() {
    let mut fixture = Fixture::new();
    let procedure = fixture.procedure.id;
    fixture.procedure.cleanups[0]
        .body
        .statements
        .push(Statement::Block(block(vec![])));
    fixture.procedure.cleanups.push(block(vec![]).into());
    let outer = fixture.root().child(0, DebugBranch::Block);
    let inner = BlockPath::cleanup(procedure, CleanupId::new(0)).child(1, DebugBranch::Block);
    fixture
        .debug
        .insert_cleanup_parent(procedure, CleanupId::new(0), outer.clone());
    fixture
        .debug
        .insert_cleanup_parent(procedure, CleanupId::new(1), inner.clone());
    fixture.debug.insert_block(
        BlockPath::cleanup(procedure, CleanupId::new(1)),
        fixture.location.clone(),
    );
    let library = fixture.finish().unwrap();
    let debug = library.debug_sources().unwrap();
    assert_eq!(
        debug.cleanup_parent(procedure, CleanupId::new(0)),
        Some(&outer)
    );
    assert_eq!(
        debug.cleanup_parent(procedure, CleanupId::new(1)),
        Some(&inner)
    );
}

#[test]
fn cleanup_lexical_parent_cannot_be_its_own_cleanup_root() {
    let mut fixture = Fixture::new();
    let procedure = fixture.procedure.id;
    fixture.debug.insert_cleanup_parent(
        procedure,
        CleanupId::new(0),
        BlockPath::cleanup(procedure, CleanupId::new(0)),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug cleanup lexical cycle",
            index: 0
        })
    ));
}

#[test]
fn cleanup_lexical_parent_cannot_cycle_between_two_cleanup_roots() {
    let mut fixture = Fixture::new();
    fixture.procedure.cleanups.push(block(vec![]).into());
    let procedure = fixture.procedure.id;
    fixture.debug.insert_cleanup_parent(
        procedure,
        CleanupId::new(0),
        BlockPath::cleanup(procedure, CleanupId::new(1)),
    );
    fixture.debug.insert_cleanup_parent(
        procedure,
        CleanupId::new(1),
        BlockPath::cleanup(procedure, CleanupId::new(0)),
    );
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug cleanup lexical cycle",
            ..
        })
    ));
}

#[test]
fn cleanup_lexical_parent_depth_includes_memoized_shallower_roots() {
    let mut fixture = Fixture::new();
    fixture.procedure.cleanups = (0..129).map(|_| block(vec![]).into()).collect();
    let procedure = fixture.procedure.id;
    fixture
        .debug
        .insert_cleanup_parent(procedure, CleanupId::new(0), fixture.root());
    for index in 1..129 {
        fixture.debug.insert_cleanup_parent(
            procedure,
            CleanupId::new(index),
            BlockPath::cleanup(procedure, CleanupId::new(index - 1)),
        );
    }
    // Earlier roots may already be memoized when a later root is visited.
    // Their full parent-chain height must still count toward the depth cap.
    assert!(matches!(fixture.finish(), Err(IrError::VerificationDepth)));
}

#[test]
fn declaration_inside_if_then_cannot_claim_procedure_root_scope() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let declaration = fixture.root().child(1, DebugBranch::IfThen).statement(0);
    let source = fixture.local_source(fixture.root(), LocalDeclaration::Statement(declaration));
    fixture.debug.insert_local(local, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug declaration scope",
            index: 0
        })
    ));
}

#[test]
fn deeply_nested_declaration_cannot_claim_any_ancestor_scope() {
    for ancestor_depth in 0..3 {
        let mut fixture = Fixture::new();
        let local = fixture.procedure.locals[0].id();
        let Statement::Block(body) = &mut fixture.procedure.body.statements[0] else {
            panic!()
        };
        for _ in 0..2 {
            let statements = std::mem::take(&mut body.statements);
            body.statements.push(Statement::Block(block(statements)));
        }
        let mut scope = fixture.root();
        for _ in 0..ancestor_depth {
            scope = scope.child(0, DebugBranch::Block);
        }
        let declaration = fixture
            .root()
            .child(0, DebugBranch::Block)
            .child(0, DebugBranch::Block)
            .child(0, DebugBranch::Block)
            .statement(0);
        let source = fixture.local_source(scope, LocalDeclaration::Statement(declaration));
        fixture.debug.insert_local(local, source);
        assert!(matches!(
            fixture.finish(),
            Err(IrError::UnknownIdentity {
                kind: "debug declaration scope",
                index: 0
            })
        ));
    }
}

#[test]
fn controlling_loop_declaration_cannot_claim_descendant_block_scope() {
    for (index, branch) in [(3, DebugBranch::While), (4, DebugBranch::Range)] {
        let mut fixture = Fixture::new();
        let local = fixture.procedure.locals[0].id();
        let body = match &mut fixture.procedure.body.statements[index] {
            Statement::While {
                body, ..
            } => body,
            Statement::Range(range) => &mut range.body,
            _ => panic!(),
        };
        let statements = std::mem::take(&mut body.statements);
        body.statements.push(Statement::Block(block(statements)));
        let scope = fixture
            .root()
            .child(index, branch)
            .child(0, DebugBranch::Block);
        let declaration = fixture.root().statement(index);
        let source = fixture.local_source(scope, LocalDeclaration::Statement(declaration));
        fixture.debug.insert_local(local, source);
        assert!(matches!(
            fixture.finish(),
            Err(IrError::UnknownIdentity {
                kind: "debug declaration scope",
                index: 0
            })
        ));
    }
}

#[test]
fn case_subject_local_declaration_uses_actual_containing_root_block() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let scope = fixture.root();
    let declaration = scope.statement(2).child(DebugBranch::CaseSubject);
    let source = fixture.local_source(
        scope.clone(),
        LocalDeclaration::Statement(declaration.clone()),
    );
    fixture.debug.insert_local(local, source);
    let library = fixture.finish().unwrap();
    let source = library.debug_sources().unwrap().local(local).unwrap();
    assert_eq!(source.scope, scope);
    assert_eq!(source.declaration, LocalDeclaration::Statement(declaration));
}

#[test]
fn case_subject_inside_if_then_cannot_claim_procedure_root_scope() {
    let mut fixture = Fixture::new();
    let local = fixture.procedure.locals[0].id();
    let cases = fixture.procedure.body.statements[2].clone();
    let Statement::If(_, then_body, _) = &mut fixture.procedure.body.statements[1] else {
        panic!()
    };
    then_body.statements = vec![cases];
    let declaration = fixture
        .root()
        .child(1, DebugBranch::IfThen)
        .statement(0)
        .child(DebugBranch::CaseSubject);
    let source = fixture.local_source(fixture.root(), LocalDeclaration::Statement(declaration));
    fixture.debug.insert_local(local, source);
    assert!(matches!(
        fixture.finish(),
        Err(IrError::UnknownIdentity {
            kind: "debug declaration scope",
            index: 0
        })
    ));
}
