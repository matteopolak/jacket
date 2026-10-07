// A call to a polymorphic procedure resolves to the instance it made, whose reported body knows
// which `#if` branch and static switch case it took (MetaThreadSafe follows only those).
// rules: records.30 records.31
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
main :: () {
    choose(true);
    pick(2);
}
choose :: ($async: bool) {
    #if async {
        safe();
    } else {
        unsafe();
    }
}
pick :: ($n: int) {
    #if n == {
        case 1; unsafe();
        case 2; safe();
    }
}
safe :: () {}
unsafe :: () {}
JAI

#run {
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);

    instance_bodies, taken_ifs, accepted_cases := 0;
    calls_to_instances := 0;
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
        if message.kind != .TYPECHECKED continue;
        typechecked := cast(*Message_Typechecked) message;
        for typechecked.procedure_bodies {
            header := it.expression.header;
            if header.name == "main" {
                for sub: it.subexpressions {
                    if sub.kind != .PROCEDURE_CALL continue;
                    callee := (cast(*Code_Procedure_Call) sub).resolved_procedure_expression;
                    if !callee || callee.kind != .PROCEDURE_HEADER continue;
                    called := cast(*Code_Procedure_Header) callee;
                    assert((called.procedure_flags & .POLYMORPHIC) == 0, "% resolves to its instance", called.name);
                    calls_to_instances += 1;
                }
            }
            if header.procedure_flags & .POLYMORPHIC continue;
            if header.name != "choose" && header.name != "pick" continue;
            instance_bodies += 1;
            for sub: it.subexpressions {
                if sub.kind != .IF continue;
                code_if := cast(*Code_If) sub;
                if !(code_if.if_flags & .IS_STATIC) continue;
                if header.name == "choose" {
                    assert((code_if.static_if_flags & .EVALUATED_AS_TRUE) != 0, "the instance took the then branch");
                    taken_ifs += 1;
                } else if code_if.if_flags & .IS_SWITCH_STATEMENT {
                    case_node := code_if.static_if_accepted_case;
                    assert(case_node != null, "the static switch names its case");
                    value := cast(*Code_Literal) case_node.condition;
                    assert(value.kind == .LITERAL && value._s64 == 2, "case 2 was taken");
                    accepted_cases += 1;
                }
            }
        }
    }
    compiler_end_intercept(w);
    assert(calls_to_instances == 2, "calls: %", calls_to_instances);
    assert(instance_bodies == 2, "instance bodies: %", instance_bodies);
    assert(taken_ifs == 1 && accepted_cases == 1, "ifs % cases %", taken_ifs, accepted_cases);
}

main :: () {
    print("ok\n");
}
