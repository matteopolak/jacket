#import "Basic";
#import "Compiler";

#run {
    modes: [3] Intercept_Flags = .[0, .SKIP_PROCEDURE_BODIES, .SKIP_EXPRESSIONS_WITHOUT_NOTES];
    for flags: modes {
        w := compiler_create_workspace("Canonical called body interception");
        options := get_build_options(w);
        options.output_type = .NO_OUTPUT;
        set_build_options(options, w);
        compiler_begin_intercept(w, flags);
        add_build_file(tprint("%interception-canonical-input.jai", #filepath), w);
        bodies: [..] *Code_Procedure_Header;
        headers: [..] *Code_Procedure_Header;
        calls: [..] *Code_Procedure_Header;
        ordinary_body, allocated_body, generic_body, complete: bool;

        while true {
            event := compiler_wait_for_message();

            assert(event != null, "completion must be delivered");

            if event.kind == .TYPECHECKED {
                message := cast(*Message_Typechecked) event;
                if flags == .SKIP_PROCEDURE_BODIES assert(message.procedure_bodies.count == 0);

                for tc: message.procedure_headers {
                    for previous: headers assert(previous != tc.expression, "duplicate canonical header event");
                    array_add(*headers, tc.expression);
                }

                for tc: message.procedure_bodies {
                    body := tc.expression;

                    assert(body && body.header && body.header.body_or_null == body);
                    for previous: bodies assert(previous != body.header, "duplicate canonical body event");

                    array_add(*bodies, body.header);
                    if flags == .SKIP_EXPRESSIONS_WITHOUT_NOTES assert(body.header.notes.count > 0);
                    if body.header.name == "ordinary" ordinary_body = true;
                    if body.header.name == "alloc" allocated_body = true;

                    if body.header.name == "generic" && !(body.header.procedure_flags & .POLYMORPHIC) {
                        generic_body = true;
                    }

                    for node: tc.subexpressions if node.kind == .PROCEDURE_CALL {
                        call := cast(*Code_Procedure_Call) node;
                        if call.resolved_procedure_expression array_add(*calls, cast(*Code_Procedure_Header) call.resolved_procedure_expression);
                    }
                }
            }

            if event.kind == .ERROR assert(false, "child workspace compilation failed");

            if event.kind == .COMPLETE {
                assert((cast(*Message_Complete) event).error_code == .NONE);
                complete = true;
                break;
            }
        }

        compiler_end_intercept(w);
        assert(complete);

        if flags == 0 {
            assert(ordinary_body && allocated_body && generic_body);
            generic_calls: int;

            for call: calls if call.name == "generic" {
                assert(!(call.procedure_flags & .POLYMORPHIC));
                assert(call.body_or_null != null);
                found := false;
                for header: bodies if header == call found = true;

                assert(found, "callee header must be the reported canonical header");
                generic_calls += 1;
            }

            assert(generic_calls == 2);
        }

        if flags == .SKIP_EXPRESSIONS_WITHOUT_NOTES assert(generic_body);

        array_free(bodies);
        array_free(headers);
        array_free(calls);
    }
}

main :: () {}
