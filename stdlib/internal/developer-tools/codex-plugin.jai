HELP_STRING :: "Codex: -export_file <path> (default exported.codex)\n";

get_plugin :: () -> *Metaprogram_Plugin {
    state := New(Codex_State);
    state.owner = context.allocator;
    state.base.init = init;
    state.base.message = message;
    state.base.finish = finish;
    state.base.shutdown = shutdown;
    state.base.communication_proc = communication_proc;
    state.base.log_help = log_help;

    return *state.base;
}

init :: (p: *Metaprogram_Plugin, options: [] string) -> bool {
    state := cast(*Codex_State) p;

    push_allocator(state.owner);
    i: s64;
    while i < options.count {
        if options[i] != "-export_file" || i + 1 >= options.count {
            log_error("Codex: expected -export_file <path>\n");
            return false;
        }

        if state.export_file.data free(state.export_file);

        state.export_file = copy_string(options[i + 1]);
        state.export_file_supplied = true;
        i += 2;
    }

    return true;
}

communication_proc :: (_p: *Metaprogram_Plugin, key: string, value: string) -> bool {
    state := cast(*Codex_State) _p;

    push_allocator(state.owner);
    destination: *string;
    if key == "project_name" destination = *state.project;
    else if key == "comment" destination = *state.comment;
    else if key == "url_or_generation_command" destination = *state.url;
    else return false;

    replacement := copy_string(value);

    if destination.data free(destination.*);

    destination.* = replacement;

    return true;
}

message :: (p: *Metaprogram_Plugin, message: *Message) {
    state := cast(*Codex_State) p;

    push_allocator(state.owner);

    if message.kind == .IMPORT array_add(*state.modules, cast(*Message_Import) message);
    else if message.kind == .FILE array_add(*state.files, cast(*Message_File) message);
    else if message.kind == .TYPECHECKED {
        for tc: (cast(*Message_Typechecked) message).procedure_bodies {
            replaced := false;

            for *old: state.bodies if old.expression == tc.expression {
                old.* = tc;
                replaced = true;
                break;
            }

            if !replaced array_add(*state.bodies, tc);
        }
    }
}

log_help :: (p: *Metaprogram_Plugin) {
    log("%", HELP_STRING);
}

finish :: (p: *Metaprogram_Plugin) {
    state := cast(*Codex_State) p;

    push_allocator(state.owner);
    encoded, ok := codex_encode(state);

    defer free(encoded);

    if !ok {
        log_error("Codex: could not encode metadata\n");
        return;
    }

    name := ifx state.export_file_supplied then state.export_file else "exported.codex";
    if !write_entire_file(name, encoded) log_error("Codex: could not write %\n", name);
}

shutdown :: (p: *Metaprogram_Plugin) {
    state := cast(*Codex_State) p;

    push_allocator(state.owner);
    free_buffers(*state.builder);
    if state.modules.data array_free(state.modules);
    if state.files.data array_free(state.files);
    if state.bodies.data array_free(state.bodies);
    if state.export_file.data free(state.export_file);
    if state.project.data free(state.project);
    if state.comment.data free(state.comment);
    if state.url.data free(state.url);

    free(state);
}

#scope_file
#import "Basic";
#import "Compiler";
#import "File";
Relative :: #import "Relative_Pointers";
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID Codex_POSIX :: #import "POSIX";
#if OS == .WINDOWS Codex_Windows :: #import "Windows";

Codex_State :: struct {
    base: Metaprogram_Plugin;
    owner: Allocator;
    export_file: string;
    export_file_supplied: bool;
    project: string;
    comment: string;
    url: string;
    modules: [..] *Message_Import;
    files: [..] *Message_File;
    bodies: [..] Typechecked(Code_Procedure_Body);
    builder: String_Builder;
}

Codex_Fixup :: struct {
    field: s64;
    target: s64;
    count: s16;
}

codex_emit_string :: (builder: *String_Builder, fixups: *[..] Codex_Fixup, offset: s64, value: string) {
    size := min(value.count, cast(s64) 32767);
    array_add(fixups, Codex_Fixup.{ offset, builder_string_length(builder), cast(s16) size });
    append(builder, value.data, size);
    append(builder, cast(u8) 0);
}

codex_timestamp :: () -> Apollo_Time, bool #no_aoc {
    seconds, nanoseconds: s64;

    #if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
        now: Codex_POSIX.timespec;
        if Codex_POSIX.clock_gettime(.REALTIME, *now) != 0 return APOLLO_TIME_INVALID, false;
        seconds = now.tv_sec;
        nanoseconds = now.tv_nsec;
    } else #if OS == .WINDOWS {
        now: Codex_Windows.FILETIME;

        Codex_Windows.GetSystemTimeAsFileTime(*now);
        ticks := (cast(u64) now.dwHighDateTime << 32) | now.dwLowDateTime;

        // Calendar difference between 1601-01-01 and 1970-01-01.
        seconds = cast(s64) (ticks / 10000000) - 11644473600;
        nanoseconds = cast(s64) (ticks % 10000000) * 100;
    } else return APOLLO_TIME_INVALID, false;

    duration := seconds_to_apollo(seconds);
    fraction := nanoseconds_to_apollo(nanoseconds);
    result := APOLLO_TIME_FOR_JAN_1_1970;
    prior := result.low;

    result.low += duration.low;
    result.high += duration.high + (ifx result.low < prior then 1 else 0);
    prior = result.low;
    result.low += fraction.low;
    result.high += fraction.high + (ifx result.low < prior then 1 else 0);

    return result, true;
}

codex_encode :: (state: *Codex_State) -> string, bool {
    free_buffers(*state.builder);
    fixups: [..] Codex_Fixup;

    defer array_free(fixups);
    builder := *state.builder;
    header: Header_Record;

    memset(*header, 0, size_of(Header_Record));
    header.magic = 0x05040003;
    header.version = 2;
    timestamp, time_ok := codex_timestamp();
    if !time_ok {
        log_error("Codex: generation clock unavailable\n");
        return "", false;
    }

    header.time_generated = timestamp;
    header.revision_control_type = .UNKNOWN;
    header.num_modules = cast(s32) state.modules.count;
    header.num_files = cast(s32) state.files.count;
    header.num_procedures = cast(s32) state.bodies.count;
    header.OS = OS;
    options := get_build_options(state.base.workspace);

    // No canonical Optimization_Type value is supplied by Build_Options.
    // Keep the initialized tag non-authoritative, as specified.
    append(builder, cast(*u8) *header, size_of(Header_Record));
    codex_emit_string(builder, *fixups, cast(s64) *header.project_name - cast(s64) *header, state.project);
    codex_emit_string(builder, *fixups, cast(s64) *header.executable_name - cast(s64) *header, options.output_executable_name);
    codex_emit_string(builder, *fixups, cast(s64) *header.comment - cast(s64) *header, state.comment);
    codex_emit_string(builder, *fixups, cast(s64) *header.url_or_generation_command - cast(s64) *header, state.url);

    for source: state.modules {
        record: Module_Record;
        record.module_type = source.module_type;
        offset := builder_string_length(builder);

        append(builder, cast(*u8) *record, size_of(Module_Record));
        codex_emit_string(builder, *fixups, offset + cast(s64) *record.module_name - cast(s64) *record, source.module_name);
        codex_emit_string(builder, *fixups, offset + cast(s64) *record.fully_pathed_filename - cast(s64) *record, source.fully_pathed_filename);
    }

    for source: state.files {
        record: File_Record;
        record.module_index = -1;

        for module, i: state.modules if module == source.enclosing_import {
            record.module_index = cast(s32) i;
            break;
        }

        if record.module_index < 0 {
            log_error("Codex: missing file module metadata\n");
            return "", false;
        }

        record.from_a_string = source.from_a_string;
        offset := builder_string_length(builder);

        append(builder, cast(*u8) *record, size_of(File_Record));
        codex_emit_string(builder, *fixups, offset + cast(s64) *record.fully_pathed_filename - cast(s64) *record, source.fully_pathed_filename);
    }

    for tc: state.bodies {
        body := tc.expression;
        record: Procedure_Record;
        record.file_index = -1;

        for source, i: state.files if source == body.enclosing_load {
            record.file_index = cast(s32) i;
            break;
        }

        if record.file_index < 0 || !body.header {
            log_error("Codex: missing procedure location metadata\n");
            return "", false;
        }

        record.l0 = body.l0;
        record.c0 = body.c0;
        record.l1 = body.l1;
        record.c1 = body.c1;
        record.procedure_flags = body.header.procedure_flags;
        record.num_expressions = cast(u32) tc.subexpressions.count;
        record.metrics = collect_metrics(tc);
        offset := builder_string_length(builder);

        append(builder, cast(*u8) *record, size_of(Procedure_Record));
        codex_emit_string(builder, *fixups, offset + cast(s64) *record.name - cast(s64) *record, body.header.name);
    }

    if builder.failed return "", false;

    result := builder_to_string(builder);

    for fix: fixups {
        field := cast(*Relative.Relative_String(s16)) (result.data + fix.field);
        displacement := fix.target - fix.field - size_of(s16);

        if displacement < -32768 || displacement > 32767 {
            log_error("Codex: relative string displacement exceeds s16\n");
            free(result);
            return "", false;
        }

        field.count = fix.count;
        Relative.write_relative_pointer(s16, *field.data, result.data + fix.target);
    }

    return result, true;
}
