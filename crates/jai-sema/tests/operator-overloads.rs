//! Operator acceptance executes only the independently compiled checked IR.
use jai_modules::{ModuleGraph, SourceOverlay};
use jai_vm::{Limits, Outcome, Value};
use std::path::Path;

fn optional_original_source(relative: &str) -> Option<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    match std::fs::read_to_string(&path) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original-source operator gate: {} is absent",
                path.display()
            );
            None
        }
        Err(error) => panic!(
            "cannot read optional original source {}: {error}",
            path.display()
        ),
    }
}

fn compile(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    compile_with_options(files, &jai_sema::ResolveOptions::default())
}
fn compile_with_options(
    files: &[(&str, &str)],
    options: &jai_sema::ResolveOptions,
) -> Result<jai_ir::Program, String> {
    let mut overlay = SourceOverlay::new();
    for (path, source) in files {
        overlay
            .insert(Path::new(path), source.as_bytes().to_vec())
            .unwrap();
    }
    let graph =
        ModuleGraph::load_with_provider(Path::new(files[0].0), Default::default(), &overlay)
            .map_err(|error| error.to_string())?;
    jai_sema::resolve_graph_with_options(&graph, options, &mut jai_vm::NoEffects)
        .map_err(|error| error.render(graph.sources()))
}
fn compile_modifier(files: &[(&str, &str)]) -> Result<jai_ir::Program, String> {
    compile_with_options(
        files,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
    )
}
fn run(source: &str) -> i128 {
    run_files(&[("/operators/main.jai", source)])
}
fn run_files(files: &[(&str, &str)]) -> i128 {
    let program = compile(files).unwrap();
    run_program(&program)
}
fn run_program(program: &jai_ir::Program) -> i128 {
    let result = jai_vm::execute(program, Limits::default());
    let Outcome::Complete(values) = result.outcome else {
        panic!("operator execution failed: {:?}", result.outcome);
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("expected one integer result");
    };
    value.value()
}

#[test]
fn lexical_operator_modifier_rejection_precedes_ranking_and_body_lowering() {
    let program = compile_modifier(&[(
        "/operators/main.jai",
        r#"
        Box::struct{value:int;}
        calls:int;
        operand::()->Box{calls+=1;return .{value=4};}
        main::()->int{
            operator +::(a:$T,b:T)->T #modify{return false,"excluded";}
                {return missing_rejected_body;}
            operator +::(a:Box,b:Box)->Box{return .{value=a.value+b.value};}
            result:=operand()+operand();return result.value+calls;
        }
    "#,
    )])
    .unwrap();
    assert_eq!(run_program(&program), 10);
}

#[test]
fn lexical_operator_modifier_introduces_result_type_in_the_definition_environment() {
    let program = compile_modifier(&[
        (
            "/operators/main.jai",
            r#"
        Base::#import,file "first.jai";
        main::()->int{
            Library::#import,file "first.jai";
            operator +::(a:$T,b:int)->$R #modify{R=Library.Output;return true;}
                {return .{value=a.value+b+Library.bias};}
            {Library::#import,file "second.jai";return (Base.Box.{value=4}+3).value;}
        }
    "#,
        ),
        (
            "/operators/first.jai",
            "Box::struct{value:int;}Output::Box;bias::5;",
        ),
        ("/operators/second.jai", "Output::bool;bias::100;"),
    ])
    .unwrap();
    assert_eq!(run_program(&program), 12);
}

#[test]
fn lexical_operator_modifier_rechecks_the_original_nominal_operand() {
    let error = compile_modifier(&[(
        "/operators/main.jai",
        r#"
        Box::struct{value:int;}
        Other::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:int)->T #modify{T=Other;return true;}
                {return .{value=a.value+b};}
            return Box.{value=4}+3;
        }
    "#,
    )])
    .unwrap_err();
    assert!(
        error.contains("operator") || error.contains("argument"),
        "{error}"
    );
}

#[test]
fn lexical_operator_modifier_reports_its_reason_and_isolates_global_writes() {
    let error = compile_modifier(&[(
        "/operators/main.jai",
        r#"
        Box::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:int)->int #modify{return false,"lexical rejection";}
                {return a.value+b;}
            return Box.{value=4}+3;
        }
    "#,
    )])
    .unwrap_err();
    assert!(error.contains("lexical rejection"), "{error}");
    let program = compile_modifier(&[(
        "/operators/main.jai",
        r#"
        Box::struct{value:int;}
        effects:int;
        main::()->int{
            operator +::(a:$T,b:int)->T #modify{effects=1;return true;}
                {return .{value=a.value+b};}
            return (Box.{value=4}+3).value+effects;
        }
    "#,
    )])
    .unwrap();
    assert_eq!(run_program(&program), 7);
}

#[test]
fn compile_time_record_operator_requests_its_actual_specialized_body() {
    let program = compile_modifier(&[(
        "/operators/main.jai",
        r#"
        calculate::()->int{
            Box::struct{
                value:int;
                operator +::(a:$T,b:int,scale:int=2)->T
                    {return .{value=a.value+b*scale};}
                evaluate::()->int{return (Box.{value=3}+5).value;}
            }
            return Box.evaluate();
        }
        answer::#run calculate();
        main::()->int{return answer;}
    "#,
    )])
    .unwrap();
    assert_eq!(run_program(&program), 13);
}

#[test]
fn local_optional_operands_do_not_downgrade_failed_constant_recipes_to_runtime() {
    for operand in ["cast(u8)300", "#run operand()"] {
        let source = format!(
            "Box::struct{{value:int;}}operand::()->u8{{return 3;}}\
             main::()->int{{operator +::(a:$T,$$b:u8)->T{{return a;}}\
             return (Box.{{value=4}}+({operand})).value;}}"
        );
        let error = compile(&[("/operators/main.jai", &source)]).unwrap_err();
        assert!(error.contains("constant materialization"), "{error}");
    }
}

#[test]
fn local_generic_operator_defaults_keep_definition_site_runtime_reads() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        factor:int=2;
        operand::()->int{factor=3;return 2;}
        main::()->int{
            operator +=::(a:*$T,b:int,scale:int=factor){a.value+=b*scale;}
            value:=Box.{value=1};value+=operand();return value.value;
        }
    "#),
        7
    );
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"
        Box::struct{value:int;}
        main::()->int{
            Library::#import,file "first.jai";
            operator +=::(a:*$T,b:int,scale:int=Library.factor){a.value+=b*scale;}
            {Library::#import,file "second.jai";value:=Box.{value=1};value+=2;return value.value;}
        }
    "#
            ),
            ("/operators/first.jai", "factor:int=2;"),
            ("/operators/second.jai", "factor:int=100;"),
        ]),
        5
    );
}

#[test]
fn arithmetic_comparison_symmetric_and_compound_operators_share_nominal_calls() {
    assert_eq!(
        run(include_str!(
            "../../../tests/corpus/positive/operator-overloads.jai"
        )),
        29
    );
}

#[test]
fn authored_floating_record_operators_keep_nominal_types() {
    assert_eq!(
        run(include_str!(
            "../../../tests/corpus/positive/operator-record-floats.jai"
        )),
        24
    );
}

#[test]
fn indexed_updates_use_getter_setter_precedence_and_capture_effects_once() {
    assert_eq!(
        run(include_str!(
            "../../../tests/corpus/positive/operator-index-updates.jai"
        )),
        11
    );
}

#[test]
fn indexed_updates_snapshot_value_receivers_before_index_effects() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        storage:Box;
        key::()->int{storage.value=100;return 1;}
        operator []::(box:Box,index:int)->int{return box.value+index;}
        operator []=::(box:*Box,index:int,value:int){box.value=value;}
        main::()->int{storage.value=10;storage[key()]+=3;return storage.value;}
    "#),
        14
    );
}

#[test]
fn local_indexed_updates_invoke_binary_overloads_for_record_elements() {
    assert_eq!(
        run(r#"
        Item::struct{value:int;}
        Box::struct{item:Item;}
        main::()->int{
            operator []::(box:Box,index:int)->Item{return box.item;}
            operator []=::(box:*Box,index:int,item:Item){box.item=item;}
            operator *::(item:Item,scale:int)->Item{return Item.{value=item.value*scale};}
            box:Box=.{item=.{value=4}};box[0]*=3;return box.item.value;
        }
    "#),
        12
    );
}

#[test]
fn builtin_arithmetic_does_not_resolve_unused_local_operator_templates() {
    assert_eq!(
        run(r#"
        main::()->int{
            operator +::(a:$T,b:T)->T{return a;}
            return 1+2;
        }
    "#),
        3
    );
}

#[test]
fn local_generic_operators_specialize_genuine_lexical_declarations() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        Other::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:T)->T{return .{value=a.value+b.value};}
            a:=Box.{value=3}; b:=Box.{value=4}; x:=Other.{value=5}; y:=Other.{value=6};
            first:=a+b; repeated:=a+b; second:=x+y;
            return first.value+repeated.value+second.value;
        }
    "#),
        25
    );
}

#[test]
fn local_generic_symmetric_operators_preserve_source_operand_order() {
    assert_eq!(
        run(r#"
        trace:int;
        Box::struct{value:int;}
        scalar::()->int{trace=trace*10+1;return 3;}
        record::()->Box{trace=trace*10+2;return .{value=4};}
        main::()->int{
            operator *::(a:$T,b:int)->T #symmetric{return .{value=a.value*b};}
            value:=scalar()*record();
            return trace+value.value;
        }
    "#),
        24
    );
}

#[test]
fn rejected_local_generic_operator_bodies_are_not_reserved_or_lowered() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:int)->T{return .{value=a.value+b};}
            operator +::(a:$T,b:bool)->T{return nonexistent_body_name;}
            value:=Box.{value=5}+3;
            return value.value;
        }
    "#),
        8
    );
}

#[test]
fn local_generic_index_operators_keep_real_getter_setter_update_calls() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        trace:int;
        key::()->int{trace=trace*10+1;return 0;}
        operand::()->int{trace=trace*10+3;return 4;}
        main::()->int{
            operator []::(a:$T,index:int)->int{trace=trace*10+2;return a.value+index;}
            operator []=::(a:*$T,index:int,value:int){trace=trace*10+4;a.value=value+index;}
            box:=Box.{value=5};box[key()]+=operand();
            if trace!=1234 return 0;
            return box.value;
        }
    "#),
        9
    );
}

#[test]
fn local_operator_templates_preserve_distinct_outer_generic_environments() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        work::(a:$T,$bias:int)->int{
            operator +::(x:$U,y:U)->U{return .{value=x.value+y.value+bias};}
            return (a+a).value;
        }
        main::()->int{return work(Box.{value=3},1)+work(Box.{value=3},10);}
    "#),
        23
    );
}

#[test]
fn local_generic_operators_match_original_nominal_template_applications() {
    assert_eq!(
        run(r#"
        Box::struct($T:Type){value:T;}
        main::()->int{
            operator +::(a:Box($T),b:Box(T))->Box(T){return .{value=a.value+b.value};}
            a:Box(int)=.{value=5};b:Box(int)=.{value=7};return (a+b).value;
        }
    "#),
        12
    );
}

#[test]
fn local_generic_operator_bodies_use_captured_scoped_imports() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"
            Box::struct{value:int;}
            main::()->int{
                Library::#import,file "first.jai";
                operator +::(a:$T,b:T)->T{return .{value=a.value+b.value+Library.increment};}
                a:=Box.{value=3};b:=Box.{value=4};
                {Library::#import,file "second.jai";return (a+b).value;}
            }
        "#
            ),
            ("/operators/first.jai", "increment::1;"),
            ("/operators/second.jai", "increment::20;"),
        ]),
        8
    );
    let error = compile(&[
        (
            "/operators/main.jai",
            r#"
            Box::struct{value:int;}
            main::()->int{
                operator +::(a:$T,b:T)->T{return .{value=a.value+b.value+Library.increment};}
                Library::#import,file "first.jai";
                a:=Box.{value=3};b:=Box.{value=4};return (a+b).value;
            }
        "#,
        ),
        ("/operators/first.jai", "increment::1;"),
    ])
    .unwrap_err();
    assert!(error.contains("Library"), "{error}");
}

#[test]
fn local_operator_headers_preserve_captured_imported_template_origins() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"
        main::()->int{
            Library::#import,file "first.jai";
            operator +::(a:Library.Box($T),b:Library.Box(T))->Library.Box(T){
                return .{value=a.value+b.value+Library.increment};
            }
            a:Library.Box(int)=.{value=3};b:Library.Box(int)=.{value=4};
            {Library::#import,file "second.jai";return (a+b).value;}
        }
        "#
            ),
            (
                "/operators/first.jai",
                "Box::struct($T:Type){value:T;} increment::1;"
            ),
            (
                "/operators/second.jai",
                "Box::struct($T:Type){value:T;} increment::20;"
            ),
        ]),
        8
    );
}

#[test]
fn local_generic_mutating_operators_bind_defaults_through_captured_places() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        main::()->int{
            operator +=::(a:*$T,b:int,scale:=2){a.value+=b*scale;}
            box:=Box.{value=5};box+=3;return box.value;
        }
    "#),
        11
    );
}

#[test]
fn recursive_local_generic_operators_reuse_reserved_signatures() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        main::()->int{
            operator +::(a:$T,b:T)->T{
                if b.value==0 return a;
                return T.{value=a.value+1}+T.{value=b.value-1};
            }
            return (Box.{value=5}+Box.{value=2}).value;
        }
    "#),
        7
    );
}

#[test]
fn local_record_operator_groups_keep_original_method_scope_identity() {
    assert_eq!(
        run(r#"
        main::()->int{
            Box::struct{
                value:int;
                operator +::(a:Box,b:Box)->Box{return .{value=a.value+b.value};}
                operator +::(a:$T,b:int)->T{return .{value=a.value+b};}
                calculate::()->int{
                    a:=Box.{value=3};b:=Box.{value=4};return ((a+b)+5).value;
                }
            }
            return Box.calculate();
        }
    "#),
        12
    );
}

#[test]
fn local_operator_specializations_recheck_required_callback_results() {
    let declarations = r#"
        Box::struct{value:int;}
        plain::()->int{return 7;}
        required::()->int #must{return 9;}
    "#;
    let body = r#"
        operator +::(a:Box,callback:$F)->Box{callback();return a;}
        first:=Box.{value=5}+plain;
    "#;
    assert_eq!(
        run(&format!(
            "{declarations} main::()->int{{{body}return first.value;}}"
        )),
        5
    );
    let source = format!(
        "{declarations} main::()->int{{{body}\
         second:=Box.{{value=6}}+required;return first.value+second.value;}}"
    );
    let error = compile(&[("/operators/main.jai", &source)])
        .err()
        .expect("required callback result must be rejected after specialization reuse");
    assert!(
        error.contains("#must") || error.contains("required result"),
        "{error}"
    );
}

#[test]
fn optional_baked_shift_operands_keep_source_arity_and_real_runtime_projection() {
    let source = r#"
        Box::struct{value:u64;}
        trace:int;
        left::()->Box{trace=trace*10+1;return .{value=8};}
        right::()->u8{trace=trace*10+2;return 2;}
        operator <<::(a:Box,$$x:u8)->Box{
            trace=trace*10+3;
            #if is_constant(x) {return .{value=(a.value<<x)+1};}
            else {return .{value=a.value<<x};}
        }
        main::()->int{
            first:=left()<<1;second:=left()<<right();
            if trace!=13123 return 0;
            return cast(int)(first.value+second.value);
        }
    "#;
    let program = compile(&[("/operators/main.jai", source)]).unwrap();
    let mut arities = program
        .procedures()
        .iter()
        .filter_map(|procedure| {
            let signature = program
                .types()
                .procedure_definition(procedure.signature)
                .ok()?;
            let first = *signature.parameters.first()?;
            (signature.results.as_ref() == [first]
                && matches!(
                    program.types().kind(first),
                    Ok(jai_types::TypeKind::Record(_))
                ))
            .then_some(signature.parameters.len())
        })
        .collect::<Vec<_>>();
    arities.sort_unstable();
    assert_eq!(
        arities,
        [1, 2],
        "baked source operands must not become runtime slots"
    );
    assert_eq!(run(source), 49);
}

#[test]
fn baked_index_reads_use_the_checked_source_binding_projection() {
    assert_eq!(
        run(r#"
        Box::struct{value:int;}
        operator []::(a:Box,$index:int)->int{return a.value+index;}
        main::()->int{return Box.{value=5}[3];}
    "#),
        8
    );
}

#[test]
fn captured_baked_operator_operands_report_the_real_projection_boundary() {
    for source in [
        "Box::struct{value:int;} operator +=::(a:*Box,$x:int){a.value+=x;} main::()->int{box:Box;box+=3;return box.value;}",
        "Box::struct{value:int;} operator []::(a:Box,$i:int)->int{return a.value+i;} operator []=::(a:*Box,i:int,value:int){a.value=value;} main::()->int{box:Box;box[3]+=2;return box.value;}",
    ] {
        let error = compile(&[("/operators/main.jai", source)]).unwrap_err();
        assert!(
            error.contains("source-aware captured argument projection"),
            "{error}"
        );
    }
}

#[test]
fn generic_operator_specializes_the_original_nominal_application() {
    assert_eq!(
        run(r#"
        Box :: struct(T:Type) { value:T; }
        operator + :: (a:Box($T), b:Box(T)) -> Box(T) { return .{value=a.value+b.value}; }
        main :: () -> int { a:Box(int)=.{value=8}; b:Box(int)=.{value=9}; result := a+b; return result.value; }
    "#),
        17
    );
}

#[test]
fn operand_literal_uses_the_selected_candidate_context() {
    assert_eq!(
        run(r#"
        Box :: struct { value:int; }
        operator + :: (a:Box, b:Box) -> Box { return Box.{value=a.value+b.value}; }
        main :: () -> int { result:Box = .{value=3} + Box.{value=8}; return result.value; }
    "#),
        11
    );
}

#[test]
fn operator_lookup_respects_exported_and_private_imports() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"using library :: #import, file "math.jai"; main :: () -> int {
            result := library.Box.{value=3} + library.Box.{value=5}; return result.value; }"#
            ),
            (
                "/operators/math.jai",
                r#"Box :: struct { value:int; }
            operator + :: (a:Box,b:Box)->Box { return Box.{value=a.value+b.value}; }
            #scope_module;
            operator + :: (a:Box,b:int)->Box { return Box.{value=a.value+b}; }"#
            ),
        ]),
        8
    );
    let error = compile(&[
        (
            "/operators/main.jai",
            r#"using library :: #import, file "math.jai"; main :: () -> int {
            result := library.Box.{value=3} + 5; return result.value; }"#,
        ),
        (
            "/operators/math.jai",
            r#"Box :: struct { value:int; } #scope_module;
            operator + :: (a:Box,b:int)->Box { return Box.{value=a.value+b}; }"#,
        ),
    ])
    .unwrap_err();
    assert!(!error.is_empty());
}

#[test]
fn scoped_imports_publish_operator_only_exports_at_their_source_position() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"types :: #import, file "types.jai"; main :: ()->int {
            #import, file "operators.jai";
            result := types.Box.{value=3} + types.Box.{value=5}; return result.value; }"#
            ),
            ("/operators/types.jai", "Box :: struct {value:int;}"),
            (
                "/operators/operators.jai",
                r#"#scope_module; types :: #import, file "types.jai";
            #scope_export; operator + :: (a:types.Box,b:types.Box)->types.Box {
                return types.Box.{value=a.value+b.value}; }"#
            ),
        ]),
        8
    );
    let error = compile(&[
        (
            "/operators/main.jai",
            r#"types :: #import, file "types.jai"; main :: ()->int {
            result := types.Box.{value=3} + types.Box.{value=5};
            #import, file "operators.jai"; return result.value; }"#,
        ),
        ("/operators/types.jai", "Box :: struct {value:int;}"),
        (
            "/operators/operators.jai",
            r#"#scope_module; types :: #import, file "types.jai";
            #scope_export; operator + :: (a:types.Box,b:types.Box)->types.Box {
                return types.Box.{value=a.value+b.value}; }"#,
        ),
    ])
    .unwrap_err();
    assert!(!error.is_empty());
}

#[test]
fn local_operator_callables_capture_preceding_scoped_operator_imports() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"main :: ()->int {
            using library :: #import, file "math.jai";
            compute :: ()->int { result := library.Box.{value=3} + library.Box.{value=5}; return result.value; }
            return compute(); }"#
            ),
            (
                "/operators/math.jai",
                r#"Box :: struct {value:int;}
            operator + :: (a:Box,b:Box)->Box { return Box.{value=a.value+b.value}; }"#
            ),
        ]),
        8
    );
}

#[test]
fn explicit_value_compound_overload_updates_the_captured_place() {
    assert_eq!(
        run(r#"
        Box :: struct {value:int;}
        operator += :: (a:Box,b:Box)->Box { return Box.{value=a.value+b.value+10}; }
        main :: ()->int { value:=Box.{value=3}; value+=Box.{value=5}; return value.value; }
    "#),
        18
    );
}

#[test]
fn local_operator_declarations_keep_distinct_origins_and_nominal_types() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            Box :: struct { value:int; }
            operator + :: (a:Box,b:Box)->Box { return Box.{value=a.value+b.value}; }
            operator + :: (a:Box,b:int)->Box { return Box.{value=a.value+b}; }
            first := Box.{value=3} + Box.{value=4}; second := first + 5;
            return second.value;
        }
    "#),
        12
    );
}

#[test]
fn symmetric_operands_keep_source_evaluation_order() {
    assert_eq!(
        run(r#"
        trace:int;
        Box :: struct { value:int; }
        scalar :: ()->int { trace = trace*10+1; return 3; }
        record :: ()->Box { trace = trace*10+2; return Box.{value=4}; }
        operator * :: (a:Box,b:int)->Box #symmetric { return Box.{value=a.value*b}; }
        main :: ()->int { result := scalar() * record(); return trace+result.value; }
    "#),
        24
    );
}

#[test]
fn compound_target_address_and_rhs_are_each_evaluated_once() {
    assert_eq!(
        run(r#"
        calls:int;
        Box :: struct { value:int; }
        index :: ()->int { calls += 1; return 0; }
        operand :: ()->Box { calls += 10; return Box.{value=5}; }
        operator + :: (a:Box,b:Box)->Box { return Box.{value=a.value+b.value}; }
        main :: ()->int { values:[1]Box; values[0] = Box.{value=2}; values[index()] += operand();
            return calls+values[0].value; }
    "#),
        18
    );
}

#[test]
fn unary_and_index_operators_are_typed_procedure_calls() {
    assert_eq!(
        run(r#"
        Box :: struct { value:int; }
        operator - :: (a:Box)->Box { return Box.{value=-a.value}; }
        operator [] :: (a:Box,index:int)->int { return a.value+index; }
        main :: ()->int { value := -Box.{value=-9}; return value[3]; }
    "#),
        12
    );
}

#[test]
fn invalid_signatures_and_equal_candidates_report_diagnostics() {
    for source in [
        "operator + :: (a:int,b:int)->int { return 0; } main :: ()->int { return 1+2; }",
        "Box::struct {value:int;} operator == :: (a:Box,b:Box)->int {return 0;} main::()->int{return 0;}",
        "Box::struct {value:int;} operator + :: (a:Box,b:Box)->int {return 0;} main::()->int{return 0;}",
        "Box::struct {value:int;} operator + :: (a:Box,b:Box)->Box{return a;} operator + :: (a:Box,b:Box)->Box{return b;} main::()->int{value:=Box.{}+Box.{};return value.value;}",
    ] {
        assert!(
            compile(&[("/operators/main.jai", source)]).is_err(),
            "{source}"
        );
    }
}

#[test]
fn actual_math_operator_sources_execute_without_simd_reinterpretation() {
    let Some(source) =
        optional_original_source("corpus/upstream/withlang-dev--open-jai/modules/Math/module.jai")
    else {
        return;
    };
    let lines = source.lines().collect::<Vec<_>>();
    let definitions = [
        lines[5..22].join("\n"),
        lines[83..89].join("\n"),
        lines[118..136].join("\n"),
    ]
    .join("\n");
    let source = format!(
        "{definitions}\nmain :: () -> int {{
        a := Vector2.{{x=2,y=3}} + Vector2.{{x=5,y=7}};
        b := 2.0 * Vector3.{{x=1,y=2,z=3}};
        c := (Vector4.{{x=2,y=4,z=6,w=8}} / 2.0) - Vector4.{{x=0,y=1,z=1,w=1}};
        q := Quaternion.{{x=1,y=2,z=3,w=4}} * 3.0;
        return cast(int) (a.x+b.z+c.w+q.w);
    }}"
    );
    assert_eq!(run(&source), 28);
}

#[test]
fn original_object_mutation_operators_keep_storage_and_pointer_results() {
    let Some(source) = optional_original_source(
        "corpus/upstream/withlang-dev--open-jai/examples/24/24.2_overloading_object.jai",
    ) else {
        return;
    };
    let definitions = source
        .lines()
        .skip(2)
        .take(21)
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "{definitions}\nmain::()->int{{
        value:Obj; value.array=.[0,1,2,3,4,5,6,7,8,9]; value[2]=2; value*=3;
        pointer:=*value[2]; <<pointer*=2;
        return value[2]+value[1];
    }}"
    );
    assert_eq!(run(&source), 15);
}

#[test]
fn mutation_operator_operands_and_storage_targets_are_evaluated_once() {
    assert_eq!(
        run(r#"
        trace:int;
        Obj::struct {value:int;}
        index::()->int {trace+=1;return 0;}
        slot::()->int {trace+=10;return 0;}
        item::()->int {trace+=100;return 5;}
        scalar::()->int {trace+=1000;return 3;}
        operator []::(obj:Obj,i:int)->int {return obj.value+i;}
        operator []=::(obj:*Obj,i:int,item:int) {obj.value=item+i;}
        operator *[]::(obj:*Obj,i:int)->*int {return *obj.value;}
        operator *=::(obj:*Obj,scalar:int) {obj.value*=scalar;}
        main::()->int {
            objects:[1]Obj;
            objects[index()][slot()]=item();
            objects[index()]*=scalar();
            pointer:=*objects[index()][slot()]; <<pointer+=2;
            if trace!=1123 return 0;
            return objects[0][0];
        }
    "#),
        17
    );
}

#[test]
fn mutation_operator_signatures_enforce_void_and_pointer_contracts() {
    for source in [
        "Obj::struct{value:int;} operator []=::(obj:Obj,i:int,item:int){} main::()->int{return 0;}",
        "Obj::struct{value:int;} operator *=::(obj:*Obj,item:int)->Obj{return obj.*;} main::()->int{return 0;}",
        "Obj::struct{value:int;} operator *[]::(obj:*Obj,i:int)->int{return 0;} main::()->int{return 0;}",
    ] {
        assert!(
            compile(&[("/operators/main.jai", source)]).is_err(),
            "{source}"
        );
    }
}

#[test]
fn local_pointer_operators_supply_addressable_storage_for_updates() {
    assert_eq!(
        run(r#"
        Obj::struct {value:int;}
        main::()->int {
            operator *=::(obj:*Obj,item:int) {obj.value*=item;}
            operator *[]::(obj:*Obj,index:int)->*int {return *obj.value;}
            value:Obj=.{value=3};
            value*=4; value[0]+=2;
            pointer:=*value[0]; <<pointer+=1;
            return value.value;
        }
    "#),
        15
    );
}

#[test]
fn modifier_rejection_precedes_operator_ranking_and_operand_lowering() {
    let source = r#"
        Obj::struct {value:int;}
        trace:int;
        operand::()->Obj {trace+=1;return Obj.{value=6};}
        operator +::(a:$T,b:T)->T #modify {return false,"excluded";} {return a;}
        operator +::(a:Obj,b:Obj)->Obj {return Obj.{value=a.value+b.value};}
        main::()->int {value:=operand()+operand();return value.value+trace;}
    "#;
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/operators/main.jai");
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = result.outcome else {
        panic!("{:?}", result.outcome)
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!()
    };
    assert_eq!(value.value(), 14);
}

#[test]
fn unrelated_record_getters_preserve_builtin_pointer_indexing() {
    assert_eq!(
        run(r#"
        Obj::struct {value:int;}
        operator []::(obj:Obj,index:int)->int {return obj.value+index;}
        main::()->int {values:[2]Obj;values[1].value=19;pointer:=*values[0];return pointer[1].value;}
    "#),
        19
    );
}

#[test]
fn operator_trailing_defaults_follow_operands_in_symmetric_and_captured_calls() {
    assert_eq!(
        run(r#"
        Obj::struct {value:int;}
        operator *::(obj:Obj,item:int,tag:int=7)->Obj #symmetric {return Obj.{value=obj.value*item+tag};}
        main::()->int {
            operator +=::(obj:*Obj,item:Obj,tag:int=9) {obj.value+=item.value+tag;}
            value:=2*Obj.{value=3}; value+=Obj.{value=4};
            return value.value;
        }
    "#),
        26
    );
    assert!(compile(&[("/operators/main.jai","Obj::struct{value:int;} operator +::(a:int,b:int,extra:Obj=.{value=1})->int{return a+b;} main::()->int{return 0;}")]).is_err());
}

#[test]
fn mutating_operator_caller_location_starts_at_the_target() {
    let source = "Source_Code_Location::struct{fully_pathed_filename:string;line_number:s64;character_number:s64;}\nObj::struct{value:int;}\noperator +=::(obj:*Obj,item:int,loc:=#caller_location){obj.value=loc.character_number;}\nmain::()->int{value:Obj; value+=4; return value.value;}";
    let mut overlay = SourceOverlay::new();
    let path = Path::new("/operators/main.jai");
    overlay.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = ModuleGraph::load_with_provider(path, Default::default(), &overlay).unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let result = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = result.outcome else {
        panic!("{:?}", result.outcome)
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!()
    };
    let offset = source.rfind("value+=4").unwrap();
    let column = source[..offset]
        .rsplit('\n')
        .next()
        .unwrap()
        .chars()
        .count()
        + 1;
    assert_eq!(value.value(), column as i128);
}

#[test]
fn operator_lookup_uses_the_nearest_lexical_token_scope() {
    assert_eq!(
        run(r#"
        Obj::struct {value:int;}
        operator +::(a:Obj,b:Obj)->Obj{return Obj.{value=a.value+b.value};}
        main::()->int {
            a:=Obj.{value=2}; b:=Obj.{value=3}; result:int;
            {operator +::(a:Obj,b:Obj)->Obj{return Obj.{value=10};} inner:=a+b;result=inner.value;}
            outer:=a+b;return result+outer.value;
        }
    "#),
        15
    );
}

#[test]
fn inequality_falls_back_to_equality_once_and_respects_explicit_overloads() {
    assert_eq!(
        run(r#"
        Obj::struct {value:int;}
        calls:int;
        operand::()->Obj {calls+=1;return Obj.{value=6};}
        operator ==::(a:Obj,b:Obj)->bool{return a.value==b.value;}
        main::()->int {
            different:=operand()!=operand(); result:int;
            if !different result=10;
            {operator !=::(a:Obj,b:Obj)->bool{return true;} if Obj.{}!=Obj.{} result+=20;}
            if Obj.{}!=Obj.{} result+=100;
            return result+calls;
        }
    "#),
        32
    );
}

#[test]
fn namespace_imports_keep_operators_in_their_namespace() {
    let library = "Obj::struct{value:int;} operator +::(a:Obj,b:Obj)->Obj{return Obj.{value=a.value+b.value};}";
    for source in [
        "library::#import,file \"math.jai\"; main::()->int{a:library.Obj=.{value=2};b:library.Obj=.{value=3};value:=a+b;return value.value;}",
        "main::()->int{library::#import,file \"math.jai\";a:library.Obj=.{value=2};b:library.Obj=.{value=3};value:=a+b;return value.value;}",
    ] {
        assert!(
            compile(&[
                ("/operators/main.jai", source),
                ("/operators/math.jai", library)
            ])
            .is_err()
        );
    }
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                "using library::#import,file \"math.jai\";main::()->int{a:library.Obj=.{value=2};b:library.Obj=.{value=3};value:=a+b;return value.value;}"
            ),
            ("/operators/math.jai", library)
        ]),
        5
    );
}

#[test]
fn imported_unary_declarations_share_the_lexical_operator_token_scope() {
    let types = "Obj::struct{value:int;}";
    let operators = "types::#import,file \"types.jai\";operator -::(value:types.Obj)->types.Obj{return types.Obj.{value=-value.value};}";
    let source = "types::#import,file \"types.jai\";operator -::(a:types.Obj,b:types.Obj)->types.Obj{return types.Obj.{value=a.value-b.value};}main::()->int{#import,file \"operators.jai\";value:=types.Obj.{value=6}-types.Obj.{value=2};return value.value;}";
    assert!(
        compile(&[
            ("/operators/main.jai", source),
            ("/operators/types.jai", types),
            ("/operators/operators.jai", operators),
        ])
        .is_err()
    );
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                "types::#import,file \"types.jai\";main::()->int{#import,file \"operators.jai\";value:=-types.Obj.{value=6};return -value.value;}"
            ),
            ("/operators/types.jai", types),
            ("/operators/operators.jai", operators),
        ]),
        6
    );
}

#[test]
fn namespace_aliases_call_original_unary_and_binary_operator_bodies() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"
Library::#import,file "math.jai";
operator-::Library.operator-;
main::()->int{
    a:Library.Box=.{value=10}; b:Library.Box=.{value=3};
    negative:=-a; difference:=a-b;
    return difference.value-negative.value;
}
"#
            ),
            (
                "/operators/math.jai",
                r#"
Box::struct{value:int;}
operator -::(value:Box)->Box{return .{value=-value.value};}
operator -::(a:Box,b:Box)->Box{return .{value=a.value-b.value};}
"#
            ),
        ]),
        17
    );
}

#[test]
fn namespace_operator_aliases_preserve_generic_nominal_specialization() {
    assert_eq!(
        run_files(&[
            (
                "/operators/main.jai",
                r#"
Library::#import,file "math.jai";
operator+::Library.operator+;
main::()->int{
    a:Library.Box(int)=.{value=5}; b:Library.Box(int)=.{value=7};
    return (a+b).value;
}
"#
            ),
            (
                "/operators/math.jai",
                r#"
Box::struct($T:Type){value:T;}
operator +::(a:Box($T),b:Box(T))->Box(T){return .{value=a.value+b.value};}
"#
            ),
        ]),
        12
    );
}
