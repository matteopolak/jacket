// The metaprogram plugin behind `jacket test`: `jaic tests.jai +Extensions/Testing/Discovery`.
// It collects the program's procedures with a `@test` or `@bench` note, then adds a table of
// the tests and an entry point, `__jacket_test_main`, that runs them (Testing's `run_tests`).
// The program's own `main`, if it has one, stays as it is and does not run.
//
// Options:
//     -manifest FILE   write one line per test and bench: kind, name, file, line, then its
//                      notes, separated by tabs
//     -registry FILE   also write the added source to FILE, for a build that adds it itself
//                      (`jacket test --interp`, where plugins do not run)
//
// Only procedures of the main program are collected, and only those at file level outside
// `#scope_file`, since the table is added at the top level and has to name them.

get_plugin :: () -> *Metaprogram_Plugin {
    plugin := New(Discovery);
    plugin.init = init;
    plugin.add_source = add_source;
    plugin.message = message;
    plugin.log_help = log_help;

    return plugin;
}

#scope_file

ENTRY_POINT :: "__jacket_test_main";

Discovery :: struct {
    #as using base: Metaprogram_Plugin;
    manifest_path: string;
    registry_path: string;
    found: [..] Found;
    added: bool;
}

Found :: struct {
    kind: string; // "test" or "bench"
    name: string;
    file: string;
    line: s64;
    notes: [] string;
    arguments: s64;
}

init :: (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
    discovery := cast(*Discovery) plugin;
    i := 0;

    while i < options.count {
        option := options[i];
        if (option == "-manifest" || option == "-registry") && i + 1 < options.count {
            if option == "-manifest" discovery.manifest_path = options[i + 1];
            else discovery.registry_path = options[i + 1];
            i += 2;
            continue;
        }

        print("Testing/Discovery: unknown option '%'\n", option);

        return false;
    }

    return true;
}

log_help :: (plugin: *Metaprogram_Plugin) {
    print("Testing/Discovery: -manifest FILE writes the tests found; -registry FILE writes the added source.\n");
}

add_source :: (plugin: *Metaprogram_Plugin) {
    options := get_build_options(plugin.workspace);
    options.entry_point_name = ENTRY_POINT;
    set_build_options(options, plugin.workspace);
}

message :: (plugin: *Metaprogram_Plugin, m: *Message) {
    discovery := cast(*Discovery) plugin;

    if m.kind == {
        case .TYPECHECKED;
            collect(discovery, cast(*Message_Typechecked) m);
        case .PHASE;
            phase := cast(*Message_Phase) m;
            if phase.phase == .TYPECHECKED_ALL_WE_CAN && !discovery.added {
                discovery.added = true;
                finish_discovery(discovery);
            }
    }
}

kind_of :: (notes: [] *Code_Note) -> string {
    for notes {
        if it.text == "test" return "test";
        if it.text == "bench" return "bench";
    }

    return "";
}

collect :: (discovery: *Discovery, typechecked: *Message_Typechecked) {
    for typechecked.declarations {
        declaration := it.expression;
        if !declaration.expression || declaration.expression.kind != .PROCEDURE_HEADER continue;

        header := cast(*Code_Procedure_Header) declaration.expression;
        kind := kind_of(header.notes);

        if !kind continue;

        load := declaration.enclosing_load;
        if !load || !load.enclosing_import || load.enclosing_import.module_type != .MAIN_PROGRAM continue;

        file := load.fully_pathed_filename;
        line := cast(s64) declaration.l0;

        if declaration.flags & .SCOPE_FILE {
            compiler_report(file, line, declaration.c0, tprint("`%` has a @% note but is under #scope_file, where the test table cannot reach it\nhelp: move it above #scope_file", declaration.name, kind));
            continue;
        }

        if !(declaration.flags & .IS_GLOBAL) || header.procedure_flags & .POLYMORPHIC continue;

        if header.arguments.count > 1 {
            compiler_report(file, line, declaration.c0, tprint("a @% procedure takes no arguments or one `*%`", kind, ifx kind == "test" then "Test" else "Bench"));
            continue;
        }

        notes: [..] string;

        for header.notes {
            if it.text != kind array_add(*notes, copy_string(it.text));
        }

        array_add(*discovery.found, .{ kind, copy_string(declaration.name), copy_string(file), line, notes, header.arguments.count });
    }
}

// One field of the manifest: tabs and line breaks would split it.
field :: (text: string) -> string {
    copy := copy_string(text,, temp);
    for *copy {
        if it.* == #char "\t" || it.* == #char "\n" || it.* == #char "\r" it.* = #char " ";
    }

    return copy;
}

quote :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "\"");

    for text {
        c := it;
        if c == #char "\\" || c == #char "\"" append(*builder, "\\");
        append(*builder, c);
    }

    append(*builder, "\"");

    return builder_to_string(*builder,, temp);
}

finish_discovery :: (discovery: *Discovery) {
    // Tests in the order they appear, file by file.
    quick_sort(discovery.found, (a: Found, b: Found) -> s64 {
        if a.file != b.file return compare(a.file, b.file);
        return a.line - b.line;
    });
    manifest: String_Builder;

    for discovery.found {
        print_to_builder(*manifest, "%\t%\t%\t%", it.kind, it.name, field(it.file), it.line);
        for note: it.notes print_to_builder(*manifest, "\t%", field(note));
        append(*manifest, "\n");
    }

    if discovery.manifest_path && !write_entire_file(discovery.manifest_path, builder_to_string(*manifest)) {
        compiler_report(tprint("Testing/Discovery: cannot write %", discovery.manifest_path));
    }

    registry: String_Builder;

    append(*registry, "#scope_file\n__jacket_testing :: #import \"Extensions/Testing\";\n");

    for discovery.found {
        if it.kind != "test" continue;
        call := ifx it.arguments then tprint("%(t);", it.name) else tprint("%();", it.name);
        print_to_builder(*registry, "__jacket_test_% :: (t: *__jacket_testing.Test) { % }\n", it_index, call);
    }

    append(*registry, "__jacket_tests :: __jacket_testing.Test_Case.[\n");

    for discovery.found {
        if it.kind != "test" continue;
        flags := "0";
        for note: it.notes if note == "allow_leaks" flags = ".ALLOW_LEAKS";
        print_to_builder(*registry, "    .{ %, __jacket_test_%, .{ %, %, 0 }, % },\n", quote(it.name), it_index, quote(it.file), it.line, flags);
    }

    append(*registry, "];\n#scope_export\n");
    print_to_builder(*registry, "% :: () { __jacket_testing.run_tests(__jacket_tests); }\n", ENTRY_POINT);
    source := builder_to_string(*registry);
    if discovery.registry_path && !write_entire_file(discovery.registry_path, source) {
        compiler_report(tprint("Testing/Discovery: cannot write %", discovery.registry_path));
    }

    add_build_string(source, discovery.workspace);
}

#import "Basic";
#import "Compiler";
#import "File";
#import "String";
#import "Sort";
