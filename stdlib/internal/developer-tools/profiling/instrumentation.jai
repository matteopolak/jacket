HELP_STRING :: "Iprof: -csv <filename>, -min_size <integer> (default 30), -modules\n";

get_plugin :: () -> *Metaprogram_Plugin {
    state := New(Profile_Plugin_State);
    state.owner = context.allocator;
    state.base.init = init;
    state.base.before_intercept = before_intercept;
    state.base.add_source = add_source;
    state.base.message = message;
    state.base.finish = finish;
    state.base.shutdown = shutdown;
    state.base.log_help = log_help;

    return *state.base;
}

init :: (p: *Metaprogram_Plugin, options: [] string) -> bool {
    state := cast(*Profile_Plugin_State) p;

    if profile_plugin_in_use {
        log_error("Iprof: one active plugin per configured module is supported\n");
        return false;
    }

    push_allocator(state.owner);
    i: s64;
    while i < options.count {
        option := options[i];
        if option == "-modules" state.modules = true;
        else if option == "-csv" {
            if i + 1 >= options.count {
                log_error("Iprof: -csv requires a filename\n");
                return false;
            }

            i += 1;
            if state.csv.data free(state.csv);

            state.csv = copy_string(options[i]);
            state.csv_requested = true;
        } else if option == "-min_size" {
            if i + 1 >= options.count {
                log_error("Iprof: -min_size requires an integer\n");
                return false;
            }

            i += 1;
            value, ok := profile_parse_size(options[i]);
            if !ok {
                log_error("Iprof: invalid -min_size integer\n");
                return false;
            }

            state.minimum = value;
        } else {
            log_error("Iprof: unknown option %\n", option);
            return false;
        }

        i += 1;
    }

    profile_plugin_in_use = true;
    state.active = true;
    instrumented_entry_point = false;
    injected_final_profiling_code = false;

    if profile_zone_infos_owned && zone_infos.data {
        push_allocator(profile_zone_infos_owner);
        array_free(zone_infos);
    }

    zone_infos = .{};
    profile_zone_infos_owner = state.owner;
    profile_zone_infos_owned = true;

    return true;
}

before_intercept :: (p: *Metaprogram_Plugin, flags: *Intercept_Flags) {
    flags.* &= ~(.SKIP_EXPRESSIONS_WITHOUT_NOTES | .SKIP_PROCEDURE_HEADERS | .SKIP_PROCEDURE_BODIES);
}

add_source :: (p: *Metaprogram_Plugin) {
    state := cast(*Profile_Plugin_State) p;
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, TO_INSERT);
    append(*builder, "\n#scope_export\n");
    append(*builder, "__independent_profile_initialize :: () { context._Iprof.Prepend_To_Main(); context._independent_iprof_program_running = true; }\n");
    append(*builder, "__independent_profile_final :: () { context._independent_iprof_program_running = false; context._Iprof.update(true); report := context._Iprof.create_report(); ");

    if state.csv_requested {
        append(*builder, "context._Iprof.save_csv_report(report, ");
        profile_source_string(*builder, state.csv);
        append(*builder, ");");
    } else append(*builder, "context._Iprof.log_text_report(report);");

    append(*builder, " }\n");
    assert(!builder.failed, "Iprof: source allocation failed");
    source := builder_to_string(*builder);

    defer free(source);
    add_build_string(source, p.workspace, message = cast(*Message) null);
}

message :: (p: *Metaprogram_Plugin, message: *Message) {
    state := cast(*Profile_Plugin_State) p;

    push_allocator(state.owner);

    if message.kind == .TYPECHECKED {
        typed := cast(*Message_Typechecked) message;
        for tc: typed.procedure_headers if profile_helper(tc.expression.name) && !(tc.expression.procedure_flags & .MACRO) {
            compiler_make_procedure_live(p.workspace, tc.expression);
        }

        for tc: typed.procedure_bodies {
            body := tc.expression;
            if !body || !body.header continue;

            name := body.header.name;
            if name == "__independent_profile_initialize" state.initializer = body;

            if name == "__independent_profile_setup_zones" && !injected_final_profiling_code {
                assert(state.initializer != null);
                prefix := compiler_get_nodes(#code {
                    init_runtime();
                });

                profile_rename_calls(state, prefix, .["__independent_profile_setup_zones"]);
                profile_prepend(state, state.initializer, cast(*Code_Block) prefix);
                injected_final_profiling_code = true;
            }

            if !profile_eligible(state, body, tc.subexpressions.count) continue;

            entry := profile_is_entry(state, body);
            index := zone_infos.count;

            array_add(*zone_infos, Zone_Info.{ name, body.enclosing_load, body.l0 });
            prefix: *Code_Node;
            if entry prefix = compiler_get_nodes(#code {
                init_runtime();
                defer init_runtime();
                __independent_profile_guard_snapshot := context._independent_iprof_runtime_active;

                context._independent_iprof_runtime_active = __independent_profile_guard_snapshot || !context._independent_iprof_program_running;
                set_smoothing(0);
                context._independent_iprof_runtime_active = __independent_profile_guard_snapshot;
            });
            else prefix = compiler_get_nodes(#code {
                __independent_profile_guard_snapshot := context._independent_iprof_runtime_active;
                context._independent_iprof_runtime_active = __independent_profile_guard_snapshot || !context._independent_iprof_program_running;
                set_smoothing(0);
                context._independent_iprof_runtime_active = __independent_profile_guard_snapshot;
            });
            if entry profile_rename_calls(state, prefix, .["__independent_profile_initialize", "__independent_profile_final", "__independent_profile_zone"]);
            else profile_rename_calls(state, prefix, .["__independent_profile_zone"]);

            events: [..] Visit.Dumb_Value;

            Visit.get_values(prefix, *events);
            patched := false;

            for event: events if event.preorder && event.node.kind == .LITERAL {
                literal := cast(*Code_Literal) event.node;
                if literal.value_type == .NUMBER {
                    literal._s64 = index;
                    patched = true;
                }
            }

            array_free(events);
            assert(patched, "Iprof: instrumentation template lacks its zone index");
            profile_prepend(state, body, cast(*Code_Block) prefix);
            if entry instrumented_entry_point = true;
        }
    } else if message.kind == .PHASE && (cast(*Message_Phase) message).phase == .TYPECHECKED_ALL_WE_CAN {
        if instrumented_entry_point && !state.source_added {
            profile_add_zone_setup(state);
            add_hook(p.workspace, "iprof_per_frame_hook", "()", "", "");
            add_hook(p.workspace, "iprof_editor_hook", "(filename: string, line: s64)", "", "filename, line");
            state.source_added = true;
        }
    } else if message.kind == .COMPLETE && (cast(*Message_Complete) message).error_code == .NONE {
        if instrumented_entry_point assert(injected_final_profiling_code, "Iprof: zone setup was not integrated");
    }
}

add_hook :: (w: Workspace, iprof_hook_name: string, iprof_hook_args: string, user_hook_name: string, user_hook_args: string) {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_to_builder(*builder, "% :: ", iprof_hook_name);
    start: s64;
    while start < iprof_hook_args.count && (iprof_hook_args[start] == 32 || iprof_hook_args[start] == 9 || iprof_hook_args[start] == 10 || iprof_hook_args[start] == 13) start += 1;

    if start < iprof_hook_args.count && iprof_hook_args[start] == #char "(" append(*builder, iprof_hook_args);
    else {
        append(*builder, "(");
        append(*builder, iprof_hook_args);
        append(*builder, ")");
    }

    append(*builder, " { ");

    if user_hook_name {
        print_to_builder(*builder, "%(%);", user_hook_name, user_hook_args);
    }

    append(*builder, " }\n");
    assert(!builder.failed, "Iprof: hook source allocation failed");
    source := builder_to_string(*builder);

    defer free(source);
    add_build_string(source, w, message = cast(*Message) null);
}

finish :: (p: *Metaprogram_Plugin) {}

shutdown :: (p: *Metaprogram_Plugin) {
    state := cast(*Profile_Plugin_State) p;

    push_allocator(state.owner);
    for statements: state.owned_statements free(statements.data);
    for node: state.owned_nodes free(node);

    if state.owned_nodes.data array_free(state.owned_nodes);

    if state.owned_statements.data array_free(state.owned_statements);
    if state.csv.data free(state.csv);

    if state.active {
        // Public module metadata outlives the private plugin instance.
        profile_plugin_in_use = false;
    }

    free(state);
}

log_help :: (p: *Metaprogram_Plugin) {
    log("%", HELP_STRING);
}

#scope_file
#import "Basic";
#import "Compiler";
Visit :: #import "Code_Visit";

Profile_Plugin_State :: struct {
    base: Metaprogram_Plugin;
    owner: Allocator;
    csv: string;
    csv_requested: bool;
    minimum: s64 = 30;
    modules: bool;
    active: bool;
    source_added: bool;
    initializer: *Code_Procedure_Body;
    owned_statements: [..][] *Code_Node;
    owned_nodes: [..] *Code_Node;
}

profile_plugin_in_use: bool;
profile_zone_infos_owner: Allocator;
profile_zone_infos_owned: bool;

profile_helper :: (name: string) -> bool {
    if name == "iprof_per_frame_hook" || name == "iprof_editor_hook" return true;

    prefix :: "__independent_profile_";
    if name.count < prefix.count return false;
    for i: 0..prefix.count - 1 if name[i] != prefix[i] return false;

    return true;
}

profile_parse_size :: (text: string) -> s64, bool {
    i: s64;
    while i < text.count && (text[i] == 32 || text[i] == 9) i += 1;

    negative := false;

    if i < text.count && (text[i] == #char "-" || text[i] == #char "+") {
        negative = text[i] == #char "-";
        i += 1;
    }

    start := i;
    value: u64;
    limit: u64 = ifx negative then 0x8000000000000000 else 0x7fffffffffffffff;

    while i < text.count && text[i] >= #char "0" && text[i] <= #char "9" {
        digit := cast(u64) (text[i] - #char "0");
        if value > (limit - digit) / 10 return 0, false;
        value = value * 10 + digit;
        i += 1;
    }

    if i == start return 0, false;
    while i < text.count && (text[i] == 32 || text[i] == 9 || text[i] == 10 || text[i] == 13) i += 1;
    if i != text.count return 0, false;

    if negative {
        if value == 0x8000000000000000 return -9223372036854775808, true;
        return -cast(s64) value, true;
    }

    return cast(s64) value, true;
}

profile_eligible :: (state: *Profile_Plugin_State, body: *Code_Procedure_Body, expression_count: s64) -> bool {
    header := body.header;
    if !body.block || profile_helper(header.name) || (body.body_flags & .ALREADY_MODIFIED) != 0 return false;

    flags := header.procedure_flags;
    if flags & (.POLYMORPHIC | .COMPILER_GENERATED | .COMPILE_TIME_ONLY | .QUICK | .QUICK_IN_BLOCK_FORM | .TYPE_ONLY | .MACRO | .SYNTACTICALLY_MARKED_AS_NO_CONTEXT | .C_CALL | .NO_CALL | .INTRINSIC) return false;

    source := body.enclosing_load;
    if !source || !source.enclosing_import return false;

    module := source.enclosing_import;
    if module.module_type == .PRELOAD || module.module_type == .RUNTIME_SUPPORT return false;
    if module.module_name == "Iprof" || module.module_name == "Pool" || module.module_name == "Runtime_Support" || module.module_name == "internal/runtime" return false;
    if (module.module_name == "Basic" || module.module_name == "internal/basic") && (header.name == "temporary_allocator_proc" || header.name == "IsScalar") return false;
    if module.module_type != .MAIN_PROGRAM && !state.modules return false;
    if profile_is_entry(state, body) return true;

    return expression_count >= state.minimum;
}

profile_prepend :: (state: *Profile_Plugin_State, body: *Code_Procedure_Body, prefix: *Code_Block) {
    assert(prefix != null && prefix.kind == .BLOCK);
    statements: [..] *Code_Node;
    for node: prefix.statements array_add(*statements, node);
    for node: body.block.statements array_add(*statements, node);

    body.block.statements = .{ statements.count, statements.data };
    array_add(*state.owned_statements, .{ statements.count, statements.data });
    compiler_modify_procedure(state.base.workspace, body);
}

profile_source_string :: (builder: *String_Builder, value: string) {
    append(builder, cast(u8) #char "\"");

    for byte: value {
        if byte == #char "\"" || byte == #char "\\" {
            append(builder, cast(u8) #char "\\");
            append(builder, byte);
        } else if byte == 10 append(builder, "\\n");
        else if byte == 13 append(builder, "\\r");
        else if byte == 9 append(builder, "\\t");
        else append(builder, byte);
    }

    append(builder, cast(u8) #char "\"");
}

profile_add_zone_setup :: (state: *Profile_Plugin_State) {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "#scope_file\n__Independent_Profile_Basic :: #import \"Basic\";\n#scope_export\n");
    append(*builder, "__independent_profile_setup_zones :: () { prior := context._independent_iprof_runtime_active; context._independent_iprof_runtime_active = true; defer context._independent_iprof_runtime_active = prior;\n");
    #if MANUAL_MODE assert(zone_infos.count <= MAX_PROFILING_ZONES_FOR_MANUAL_MODE);
    else print_to_builder(*builder, "context._Iprof.zones = __Independent_Profile_Basic.NewArray(%, context._Iprof.Zone);\n", zone_infos.count);

    for zone, index: zone_infos {
        print_to_builder(*builder, "context._Iprof.zones[%].initted = true; context._Iprof.zones[%].hash = %; context._Iprof.zones[%].line = %;\n", index, index, index, index, zone.line);
        print_to_builder(*builder, "context._Iprof.zones[%].name = ", index);
        profile_source_string(*builder, zone.name);
        append(*builder, ";\n");
        path := ifx zone.load then zone.load.fully_pathed_filename else "";

        print_to_builder(*builder, "context._Iprof.zones[%].full_path = ", index);
        profile_source_string(*builder, path);
        append(*builder, ";\n");
        print_to_builder(*builder, "context._Iprof.zones[%].filename = ", index);
        profile_source_string(*builder, path);
        append(*builder, ";\n");
    }

    append(*builder, "}\n");
    assert(!builder.failed, "Iprof: zone setup allocation failed");
    source := builder_to_string(*builder);

    defer free(source);
    add_build_string(source, state.base.workspace, message = cast(*Message) null);
}

profile_rename_calls :: (state: *Profile_Plugin_State, root: *Code_Node, names: [] string) {
    events: [..] Visit.Dumb_Value;

    defer array_free(events);
    Visit.get_values(root, *events);
    i: s64;

    for event: events if event.preorder && event.node.kind == .PROCEDURE_CALL {
        call := cast(*Code_Procedure_Call) event.node;

        assert(call.procedure_expression && call.procedure_expression.kind == .IDENT && i < names.count);

        if names[i] == "__independent_profile_zone" {
            from := compiler_get_nodes(#code context._independent_iprof_runtime_active);

            assert(from.kind == .BINARY_OPERATOR);
            (cast(*Code_Ident) (cast(*Code_Binary_Operator) from).right).name = "_Iprof";
            member := New(Code_Ident);
            member.kind = .IDENT;
            member.name = "Automatic_Zone_Guarded";
            dot := New(Code_Binary_Operator);
            dot.kind = .BINARY_OPERATOR;
            dot.operator_type = cast(Operator_Type) #char ".";
            dot.left = from;
            dot.right = cast(*Code_Node) member;
            call.procedure_expression = cast(*Code_Node) dot;
            array_add(*state.owned_nodes, cast(*Code_Node) member);
            array_add(*state.owned_nodes, cast(*Code_Node) dot);
        } else (cast(*Code_Ident) call.procedure_expression).name = names[i];

        i += 1;
    }

    assert(i == names.count);
}

profile_is_entry :: (state: *Profile_Plugin_State, body: *Code_Procedure_Body) -> bool {
    if body.header.procedure_flags & .ENTRY_POINT return true;
    options := get_build_options(state.base.workspace);
    name := ifx options.entry_point_name then options.entry_point_name else "main";
    return body.header.name == name && body.enclosing_load != null && body.enclosing_load.enclosing_import != null && body.enclosing_load.enclosing_import.module_type == .MAIN_PROGRAM;
}
