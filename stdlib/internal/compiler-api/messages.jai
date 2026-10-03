compiler_begin_intercept :: (w: Workspace, flags: C.Intercept_Flags = 0) #compiler {
    ws := resolve_workspace(w);

    require(ws != __jaic_current_workspace(), "a workspace cannot wait for its own compilation");
    state := workspace_slot(ws);
    if !state.intercepting Basic.array_add(*intercepted, ws);

    state.intercepting = true;
    state.finished = false;
    state.intercept_flags = flags;
    __jaic_workspace_begin_intercept(ws, cast(u32) flags);
}

compiler_end_intercept :: (w: Workspace) #compiler {
    ws := resolve_workspace(w);
    workspace_slot(ws).intercepting = false;
    __jaic_workspace_end_intercept(ws);
}

compiler_set_workspace_status :: (status: C.Workspace_Status, w: Workspace = -1) #compiler {
    __jaic_workspace_status(resolve_workspace(w), cast(u8) status);
}

compiler_wait_for_message :: () -> *C.Message #compiler {
    initialize_records();
    count := intercepted.count;
    for 0..count - 1 {
        cursor = (cursor + 1) % count;
        w := intercepted[cursor];
        state := workspace_slot(w);

        if !state.intercepting || state.finished continue;

        event := __jaic_workspace_next_event(w);
        if event == 0 {
            state.finished = true;
            continue;
        }

        if event == 1 || event == 4 || event == 5 || event == 6 || event == 9 {
            result := cast(*C.Message) __jaic_rec_build(record_builder, __jaic_event_int(0), type_info(C.Message));
            require(result != null, "compiler message record could not be materialized");
            if result.kind == .TYPECHECKED filter_typechecked(cast(*C.Message_Typechecked) result, state.intercept_flags);
            return result;
        }

        if event == 2 {
            result := Basic.New(C.Message_Phase);
            result.kind = .PHASE;
            result.workspace = w;
            result.phase = cast(type_of(result.phase)) __jaic_event_int(0);
            if result.phase == .PRE_WRITE_EXECUTABLE || result.phase == .READY_FOR_CUSTOM_LINK_COMMAND {
                at: s64 = 0;
                result.compiler_generated_object_files, at = event_strings(__jaic_event_int(1), at);
                result.support_object_files, at = event_strings(__jaic_event_int(2), at);
                result.system_libraries, at = event_strings(__jaic_event_int(3), at);
                result.user_libraries, at = event_strings(__jaic_event_int(4), at);
                __jaic_event_string(at, *result.executable_name);
            } else if result.phase == .POST_WRITE_EXECUTABLE {
                result.executable_write_failed = __jaic_event_int(1) != 0;
                result.linker_exit_code = cast(s32) __jaic_event_int(2);
                __jaic_event_string(0, *result.executable_name);
            }

            if result.phase == .READY_FOR_CUSTOM_LINK_COMMAND state.link_phase = result;

            return result;
        }

        if event == 3 {
            result := Basic.New(C.Message_Complete);
            result.kind = .COMPLETE;
            result.workspace = w;
            result.error_code = cast(type_of(result.error_code)) __jaic_event_int(0);
            state.finished = true;
            last_complete = result;

            return result;
        }

        if event == 7 {
            result := Basic.New(C.Message);
            result.kind = .ERROR;
            result.workspace = w;
            return result;
        }

        if event == 8 {
            result := Basic.New(C.Message_Debug_Dump);
            result.kind = .DEBUG_DUMP;
            result.workspace = w;
            __jaic_event_string(0, *result.dump_text);

            return result;
        }

        compiler_report("unsupported compiler message event");
    }

    return last_complete;
}

compiler_custom_link_command_is_complete :: (w: Workspace) #compiler {
    ws := resolve_workspace(w);
    state := workspace_slot(ws);

    require(state.link_phase != null, "workspace is not waiting for custom linking");
    __jaic_custom_link_complete(ws, state.link_phase.linker_exit_code);
    state.link_phase = null;
}

#scope_module
intercepted: [..] Workspace;
cursor: s64 = -1;
last_complete: *C.Message_Complete;

event_strings :: (count: s64, start: s64) -> (strings: [] string, next: s64) {
    require(count >= 0, "negative compiler event string count");
    result: [..] string;

    for i: 0..count - 1 {
        value: string;
        __jaic_event_string(start + i, *value);
        Basic.array_add(*result, value);
    }

    return result, start + count;
}

filter_typechecked :: (message: *C.Message_Typechecked, flags: C.Intercept_Flags) {
    if flags == 0 return;
    if flags & .SKIP_DECLARATIONS message.declarations = .[];
    if flags & .SKIP_PROCEDURE_HEADERS message.procedure_headers = .[];
    if flags & .SKIP_PROCEDURE_BODIES message.procedure_bodies = .[];
    if flags & .SKIP_STRUCTS message.structs = .[];
    if flags & .SKIP_OTHERS message.others = .[];

    if flags & .SKIP_EXPRESSIONS_WITHOUT_NOTES {
        declarations: [..] C.Typechecked(C.Code_Declaration);
        for message.declarations if it.expression.notes.count > 0 Basic.array_add(*declarations, it);

        message.declarations = declarations;
        headers: [..] C.Typechecked(C.Code_Procedure_Header);
        for message.procedure_headers if it.expression.notes.count > 0 Basic.array_add(*headers, it);

        message.procedure_headers = headers;
        bodies: [..] C.Typechecked(C.Code_Procedure_Body);
        for message.procedure_bodies if it.expression.header && it.expression.header.notes.count > 0 Basic.array_add(*bodies, it);

        message.procedure_bodies = bodies;
        structs: [..] C.Typechecked(C.Code_Struct);
        for message.structs if it.expression.notes.count > 0 Basic.array_add(*structs, it);

        message.structs = structs;
        others: [..] C.Typechecked(C.Code_Node);
        for message.others if it.expression.kind == .NOTE Basic.array_add(*others, it);

        message.others = others;
    }

    all: [..] C.Typechecked(C.Code_Node);
    for message.declarations Basic.array_add(*all, C.Typechecked(C.Code_Node).{ it.expression, it.subexpressions });
    for message.procedure_headers Basic.array_add(*all, C.Typechecked(C.Code_Node).{ it.expression, it.subexpressions });
    for message.procedure_bodies Basic.array_add(*all, C.Typechecked(C.Code_Node).{ it.expression, it.subexpressions });
    for message.structs Basic.array_add(*all, C.Typechecked(C.Code_Node).{ it.expression, it.subexpressions });
    for message.others Basic.array_add(*all, it);

    message.all = all;
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
