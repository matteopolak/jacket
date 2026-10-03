//! Ordinary partial applications publish and execute actual wrapper procedures.
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_types::TypeView;
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn compile(source: &str) -> Result<jai_ir::Program, String> {
    let path = Path::new("/ordinary-baked/main.jai");
    let mut overlay = SourceOverlay::new();
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay)
        .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .map_err(|error| error.render(graph.sources()))
}

fn run(source: &str) -> (jai_ir::Program, i128) {
    let program = compile(source).unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = result.outcome else {
        panic!("{result:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("{values:?}");
    };
    let answer = value.value();
    (program, answer)
}

fn real_wrappers(
    program: &jai_ir::Program,
    original_parameters: usize,
    remaining_parameters: usize,
) -> Vec<jai_ir::ProcedureId> {
    program
        .procedures()
        .iter()
        .filter_map(|procedure| {
            if procedure.parameters.len() != remaining_parameters {
                return None;
            }
            let call = match procedure.body.statements.first()? {
                jai_ir::Statement::CallResults {
                    call, ..
                }
                | jai_ir::Statement::CallVoid(call) => call,
                _ => return None,
            };
            let original = program.procedure_by_id(call.procedure)?;
            (original.id != procedure.id
                && original.parameters.len() == original_parameters
                && call.arguments.len() == original_parameters
                && call
                    .arguments
                    .iter()
                    .enumerate()
                    .all(|(ordinal, (parameter, _))| parameter.index() == ordinal))
            .then_some(procedure.id)
        })
        .collect()
}

#[test]
fn ordinary_bakes_create_real_wrappers_and_keep_nontrailing_slots() {
    let (program, answer) = run(
        "sum::(a:int,b:int,c:int)->int{return a+b+c;} main::()->int {f::#bake_arguments sum(b=2); return f(20,20);}",
    );
    assert_eq!(answer, 42);
    let wrappers = real_wrappers(&program, 3, 2);
    assert_eq!(wrappers.len(), 1);
    let wrapper = program.procedure_by_id(wrappers[0]).unwrap();
    let jai_ir::Statement::CallResults {
        call, ..
    } = &wrapper.body.statements[0]
    else {
        panic!("wrapper contains its genuine original call");
    };
    assert_eq!(
        call.arguments
            .iter()
            .map(|(parameter, _)| parameter.index())
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(
        program
            .procedure_by_id(call.procedure)
            .unwrap()
            .parameters
            .len(),
        3
    );
}

#[test]
fn floating_bakes_preserve_the_declared_type_and_definition_scope() {
    let (_, answer) = run(
        "Bias::9; mult::(a:float,b:float)->float{return a*b;} baked::#bake_arguments mult(b=-Bias); main::()->int {Bias::100; return cast(int)baked(-6);}",
    );
    assert_eq!(answer, 54);
}

#[test]
fn baked_type_formals_select_the_genuine_generic_body() {
    let (program, answer) = run(
        "add::(value:T,$T:Type)->T {return value+2;} main::()->int {f::#bake_arguments add(T=int); return f(40);}",
    );
    assert_eq!(answer, 42);
    assert!(
        real_wrappers(&program, 2, 1).is_empty(),
        "an already-erased baked formal needs no runtime wrapper"
    );
}

#[test]
fn named_defaults_and_result_obligations_survive_wrapper_metadata() {
    let (_, answer) = run(
        "sum::(a:int,b:int,c:int=20)->int{return a+b+c;} main::()->int {callback:=#bake_arguments sum(b=2); return callback(a=20);}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn duplicate_unknown_and_runtime_bakes_are_located_errors() {
    for (source, message) in [
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){f::#bake_arguments sum(b=1,b=2);}",
            "duplicate",
        ),
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){f::#bake_arguments sum(c=1);}",
            "unknown baked",
        ),
        (
            "sum::(a:int,b:int)->int{return a+b;}main::(){value:int=1;f::#bake_arguments sum(b=value);}",
            "constant",
        ),
    ] {
        let error = compile(source).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn c_callback_type_baking_keeps_the_actual_convention_and_body() {
    let (program, answer) = run(
        "worker::(value:int,$User_Data:Type)->int #c_call{return value+size_of(User_Data);} main::()->int {f:=#bake_arguments worker(User_Data=u8);return f(41);}",
    );
    assert_eq!(answer, 42);
    assert!(real_wrappers(&program, 2, 1).is_empty());
    assert!(program.procedures().iter().any(|procedure| {
        let signature = program
            .types()
            .procedure_definition(procedure.signature)
            .unwrap();
        signature.convention == jai_types::CallingConvention::C && signature.parameters.len() == 1
    }));
}

#[test]
fn a_real_wrapper_forwards_the_current_implicit_context() {
    let (_, answer) = run(
        "#add_context Bias:int=0; target::(value:int,bias:int)->int{return value+bias+context.Bias;} main::()->int{context.Bias=40;f:=#bake_arguments target(bias=1);return f(1);}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn per_use_returned_callback_policies_share_one_real_wrapper() {
    let prefix = "Optional::#type(value:int=21)->int;Required::#type(value:int=21)->int #must;answer::(input:int)->int{return input;}forward::(tag:int,callback:$F)->F{return callback;}";
    let body = "optional_factory::#bake_arguments forward(callback=cast(Optional)answer);required_factory::#bake_arguments forward(callback=cast(Required)answer);optional:=optional_factory(0);required:=required_factory(1);optional(value=0);";
    let accepted = format!("{prefix}main::()->int{{{body}return required()+21;}}");
    let (program, answer) = run(&accepted);
    assert_eq!(answer, 42);
    let wrappers = real_wrappers(&program, 2, 1);
    assert_eq!(wrappers.len(), 1, "one genuine shared wrapper");
    assert!(program.checked_procedure(wrappers[0]).is_some());
    let rejected = format!("{prefix}main::()->int{{{body}required();return 0;}}");
    let error = compile(&rejected).unwrap_err();
    assert!(error.contains("#must"), "{error}");
}

#[test]
fn strengthening_a_bound_callback_is_a_terminal_body_error() {
    let source = "Optional::#type(value:int)->int;Required::#type(value:int)->int #must;answer::(input:int)->int{return input;}ignore::(tag:int,callback:$F)->int{callback(value=tag);return 21;}main::()->int{first::#bake_arguments ignore(callback=cast(Optional)answer);second::#bake_arguments ignore(callback=cast(Required)answer);return first(0)+second(0);}";
    let error = compile(source).unwrap_err();
    assert!(error.contains("#must"), "{error}");
}

#[test]
fn strengthening_a_reused_generic_body_refreshes_its_local_baked_constant() {
    let source = "Optional::#type(value:int)->int;Required::#type(value:int)->int #must;answer::(input:int)->int{return input;}forward::(callback:$F)->F{return callback;}outer::(callback:$F)->int{factory::#bake_arguments forward(callback=cast(F)answer);retained:=factory();retained(value=0);return 21;}main::()->int{return outer(cast(Optional)answer)+outer(cast(Required)answer);}";
    let error = compile(source).unwrap_err();
    assert!(error.contains("#must"), "{error}");
}

#[test]
fn callee_cast_names_defaults_and_obligations_share_the_original_wrapper() {
    let prefix = "Optional::#type(fixed:int,value:int=21)->int;Required::#type(fixed:int,number:int=21)->int #must;target::(bias:int,input:int)->int{return bias+input;}";
    let body = "optional_target::cast(Optional)target;required_target::cast(Required)target;optional::#bake_arguments optional_target(fixed=0);required::#bake_arguments required_target(fixed=0);optional(value=0);";
    let (program, answer) = run(&format!(
        "{prefix}main::()->int{{{body}return optional()+required(number=21);}}"
    ));
    assert_eq!(answer, 42);
    let wrappers = real_wrappers(&program, 2, 1);
    assert_eq!(wrappers.len(), 1, "one genuine shared wrapper");
    assert!(program.checked_procedure(wrappers[0]).is_some());
    let error = compile(&format!(
        "{prefix}main::()->int{{{body}required();return 0;}}"
    ))
    .unwrap_err();
    assert!(error.contains("#must"), "{error}");
    let error = compile(&format!(
        "{prefix}main::()->int{{{body}return required(value=21);}}"
    ))
    .unwrap_err();
    assert!(
        error.contains("unknown") || error.contains("name"),
        "{error}"
    );
}

#[test]
fn runtime_named_arguments_run_once_in_written_order() {
    let (_, answer) = run(
        "Counter:int=0; next::(digit:int)->int{Counter=Counter*10+digit;return digit;} target::(a:int,b:int,c:int)->int{return a+b+c;} main::()->int{f::#bake_arguments target(b=40); result:=f(c=next(1),a=next(2));if Counter!=12 return 0;return result-1;}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn all_baked_and_empty_bakes_keep_real_original_calls() {
    let (program, answer) = run(
        "target::(value:int)->int{return value;} main::()->int{closed::#bake_arguments target(value=21);open::#bake_arguments target();return closed()+open(21);}",
    );
    assert_eq!(answer, 42);
    assert_eq!(real_wrappers(&program, 1, 0).len(), 1);
    assert_eq!(real_wrappers(&program, 1, 1).len(), 1);
}

#[test]
fn local_target_uses_its_defining_constant_and_original_body() {
    let (program, answer) = run(
        "main::()->int{Bias::20;target::(a:int,b:int)->int{return a+b+Bias;} f::#bake_arguments target(b=1);return f(21);}",
    );
    assert_eq!(answer, 42);
    assert_eq!(real_wrappers(&program, 2, 1).len(), 1);
}

#[test]
fn multiple_results_are_returned_in_original_order() {
    let (program, answer) = run(
        "target::(value:int,bias:int)->(int,int){return value+bias,value-bias;} main::()->int{f::#bake_arguments target(bias=1);a,b:=f(21);return a+b;}",
    );
    assert_eq!(answer, 42);
    let wrappers = real_wrappers(&program, 2, 1);
    assert_eq!(wrappers.len(), 1);
    assert_eq!(
        program
            .types()
            .procedure_definition(program.procedure_by_id(wrappers[0]).unwrap().signature)
            .unwrap()
            .results
            .len(),
        2
    );
}

#[test]
fn void_wrapper_preserves_the_original_effect() {
    let (_, answer) = run(
        "Answer:int=0;target::(value:int,bias:int){Answer=value+bias;} main::()->int{f::#bake_arguments target(bias=1);f(41);return Answer;}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn typed_local_and_file_constants_keep_callback_defaults() {
    let (_, answer) = run(
        "Partial::#type(value:int=21)->int;target::(bias:int,value:int)->int{return bias+value;}global:Partial:#bake_arguments target(bias=0);main::()->int{local:Partial:#bake_arguments target(bias=0);return global()+local();}",
    );
    assert_eq!(answer, 42);
}

#[test]
fn baked_argument_source_order_does_not_change_canonical_identity() {
    let (program, answer) = run(
        "target::(a:int,b:int,c:int)->int{return a+b+c;}main::()->int{first::#bake_arguments target(a=1,c=20);second::#bake_arguments target(c=20,a=1);return first(0)+second(0);}",
    );
    assert_eq!(answer, 42);
    assert_eq!(real_wrappers(&program, 3, 1).len(), 1);
}

#[test]
fn variadic_and_discard_bakes_fail_before_producing_a_wrapper() {
    for (source, expected) in [
        (
            "target::(fixed:int,values:..int)->int{return fixed;}main::(){f::#bake_arguments target(fixed=1);}",
            "variadic",
        ),
        (
            "target::(#discard ignored:int,value:int)->int{return value;}main::(){f::#bake_arguments target(ignored=1);}",
            "discarded",
        ),
    ] {
        let error = compile(source).unwrap_err();
        assert!(error.contains(expected), "{error}");
    }
}
