#import "Basic";
#import "Compiler";
#import "File";
C :: #import "Codex";
R :: #import "Relative_Pointers";

diagnostics: s64;

capture :: (text: string, data: *void, info: Log_Info) {
    if info.common_flags & .ERROR diagnostics += 1;
}

#run {
    push_context {
        context.logger = capture;
        bad := C.get_plugin();

        assert(!C.init(bad, .["-unknown"]));
        C.shutdown(bad);
        bad = C.get_plugin();
        assert(!C.init(bad, .["-export_file"]));
        C.shutdown(bad);
    }

    assert(diagnostics == 2);
    p := C.get_plugin();
    p.workspace = compiler_create_workspace("Original Codex lifecycle");
    options := get_build_options(p.workspace);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, p.workspace);
    destination := tprint("%../../target/profiler-codex-lifecycle.codex", #filepath);

    assert(C.init(p, .["-export_file", destination]));
    project := copy_string("project héllo");

    assert(C.communication_proc(p, "project_name", project));
    memset(project.data, #char "X", project.count);
    free(project);
    assert(C.communication_proc(p, "comment", "original\ncomment"));
    assert(C.communication_proc(p, "url_or_generation_command", "native build"));
    assert(!C.communication_proc(p, "unknown", "rejected"));
    compiler_begin_intercept(p.workspace);
    add_build_file(tprint("%codex-plugin-input.jai", #filepath), p.workspace);
    completed := false;
    conditional_expressions: u32;
    received_bodies: [..] *Code_Procedure_Body;

    defer array_free(received_bodies);

    while true {
        event := compiler_wait_for_message();
        if !event break;

        C.message(p, event);

        if event.kind == .TYPECHECKED {
            C.message(p, event);

            for tc: (cast(*Message_Typechecked) event).procedure_bodies {
                seen := false;
                for body: received_bodies if body == tc.expression seen = true;
                if !seen array_add(*received_bodies, tc.expression);
                if tc.expression.header && tc.expression.header.name == "conditional" conditional_expressions = cast(u32) tc.subexpressions.count;
            }
        }

        if event.kind == .COMPLETE {
            assert((cast(*Message_Complete) event).error_code == .NONE);
            completed = true;
            break;
        }
    }

    compiler_end_intercept(p.workspace);
    assert(completed);
    C.finish(p);

    push_context {
        context.logger = capture;
        missing := tprint("%../../target/profiler-codex-absent-parent/export.codex", #filepath);
        assert(C.init(p, .["-export_file", missing]));
        C.finish(p);
    }

    assert(diagnostics > 2);
    prior_diagnostics := diagnostics;

    push_context {
        context.logger = capture;
        assert(C.init(p, .["-export_file", ""]));
        C.finish(p);
    }

    assert(diagnostics > prior_diagnostics);
    C.shutdown(p);
    memory, read_ok := read_entire_file(destination);

    assert(read_ok);
    defer free(memory);
    ok, help, error := C.make_help(.{ memory.count, memory.data });

    assert(ok, "%", error);
    defer C.deinit(*help);
    assert(received_bodies.count >= 3);
    assert(help.procedures.count == received_bodies.count, "serialized %, received % unique bodies", help.procedures.count, received_bodies.count);
    assert(conditional_expressions > 0);
    for record: help.procedures if R.decode(*record.name) == "conditional" assert(record.num_expressions == conditional_expressions, "serialized %, compiler %", record.num_expressions, conditional_expressions);

    assert(R.decode(*help.header.project_name) == "project héllo");
    assert(R.decode(*help.header.comment) == "original\ncomment");
    assert(R.decode(*help.header.url_or_generation_command) == "native build");
    assert(help.header.time_generated.high != APOLLO_TIME_INVALID.high);
}

main :: () {
    print("ok\n");
}
