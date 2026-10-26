// The metaprogram plugin behind `jacket test`: `jaic tests.jai +Extensions/Testing/Discovery`.
// It collects the program's procedures with a `@test` or `@bench` note, then adds a table of
// the tests and an entry point, `__jacket_test_main`, that runs them (Testing's `run_tests`).
// The program's own `main`, if it has one, stays as it is and does not run.
//
// Its one option, `list`, is for a program that is only listed (`jacket test --list`): the
// `@compile_time` tests are named but not run, since nothing will replay what they report.
// It prints what it found on standard output as records, the framing the
// test program's own records use (`\x02`, the kind, each field after `\x1f`, then `\x03`):
//     manifest    one per test and bench: kind, name, file, line, then its notes
//     source      one per line of the code it adds, for a program that adds it itself, since
//                 `-interp` compiles much less of a program than a build with a plugin does
//                 (`jacket test --interp`)
//     discovered  last: everything above is out
// Printing works for a target whose compile-time code cannot write files (`-os wasm`).
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
CASES_NOTE :: "cases(";
PROPERTY_NOTE :: "property";
GENERATE_NOTE :: "generate(";

Discovery :: struct {
    #as using base: Metaprogram_Plugin;
    found: [..] Found;
    added: bool;
    list: bool; // the `list` option
}

Found :: struct {
    kind: string; // "test" or "bench"
    name: string;
    file: string;
    line: s64;
    notes: [] string;
    arguments: s64;
    cases: string; // the list of a `@cases(LIST)` note
    property: bool; // a `@property` test
    trials: string; // the count of `@property(N)`, or nothing
    generators: [] string; // the procedures of `@generate(NAME)` notes
    compile_time: bool; // a `@compile_time` test
}

init :: (plugin: *Metaprogram_Plugin, options: [] string) -> bool {
    for options {
        if it == "list" {
            (cast(*Discovery) plugin).list = true;
        } else {
            print("Testing/Discovery: unknown option '%'\n", it);

            return false;
        }
    }

    return true;
}

log_help :: (plugin: *Metaprogram_Plugin) {
    print("Testing/Discovery: takes the option `list` (name the compile-time tests without running them); it prints the tests it found and the code it adds, for jacket test.\n");
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

// The list of a `@cases(LIST)` note, or nothing.
cases_of :: (notes: [] *Code_Note) -> string {
    for notes {
        if begins_with(it.text, CASES_NOTE) && ends_with(it.text, ")") {
            return string.{ it.text.count - CASES_NOTE.count - 1, it.text.data + CASES_NOTE.count };
        }
    }

    return "";
}

has_note :: (notes: [] *Code_Note, text: string) -> bool {
    for notes if it.text == text return true;

    return false;
}

// Whether there is a `@property` note, and its count if it has one.
property_of :: (notes: [] *Code_Note) -> (found: bool, trials: string) {
    for notes {
        if it.text == PROPERTY_NOTE return true, "";

        if begins_with(it.text, "property(") && ends_with(it.text, ")") {
            return true, string.{ it.text.count - "property(".count - 1, it.text.data + "property(".count };
        }
    }

    return false, "";
}

generators_of :: (notes: [] *Code_Note) -> [] string {
    names: [..] string;

    for notes {
        if begins_with(it.text, GENERATE_NOTE) && ends_with(it.text, ")") {
            array_add(*names, copy_string(string.{ it.text.count - GENERATE_NOTE.count - 1, it.text.data + GENERATE_NOTE.count }));
        }
    }

    return names;
}

// What is wrong with the `@samples(N)` and `@min_time(MS)` notes of a bench, if anything.
bench_note_problem :: (notes: [] *Code_Note) -> string {
    for notes {
        for prefix: string.["samples(", "min_time("] {
            if !begins_with(it.text, prefix) continue;

            digits := string.{ it.text.count - prefix.count - 1, it.text.data + prefix.count };
            count, ok := string_to_int(digits);

            if !ends_with(it.text, ")") || !ok || count < 1 {
                return tprint("@% takes a positive whole number, as in `@%100)`, not `%`", string.{ prefix.count - 1, prefix.data }, prefix, digits);
            }
        }
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

        cases := ifx kind == "test" then cases_of(header.notes) else "";

        property, trials := property_of(header.notes);
        compile_time := kind == "test" && has_note(header.notes, "compile_time");

        if kind != "test" property = false;

        if compile_time && (property || cases) {
            compiler_report(file, line, declaration.c0, "a @compile_time test cannot have @property or @cases: it runs once, in the compiler");
            continue;
        }

        if property && cases {
            compiler_report(file, line, declaration.c0, "a @test cannot have both @property and @cases: a property generates its arguments");
            continue;
        }

        if property && trials {
            count, ok := string_to_int(trials);
            if !ok || count < 1 {
                compiler_report(file, line, declaration.c0, tprint("@property takes a whole number of trials, as in `@property(500)`, not `%`", trials));
                continue;
            }
        }

        if kind == "bench" {
            if bench_note_problem(header.notes) {
                compiler_report(file, line, declaration.c0, bench_note_problem(header.notes));
                continue;
            }
        }

        if header.arguments.count > 1 && !cases && !property {
            compiler_report(file, line, declaration.c0, tprint("a @% procedure takes no arguments or one `*%` (a @test with `@cases(LIST)` takes the entry as its arguments)", kind, ifx kind == "test" then "Test" else "Bench"));
            continue;
        }

        notes: [..] string;

        for header.notes {
            if it.text != kind array_add(*notes, copy_string(it.text));
        }

        array_add(*discovery.found, .{ kind, copy_string(declaration.name), copy_string(file), line, notes, header.arguments.count, copy_string(cases), property, copy_string(trials), generators_of(header.notes), compile_time });
    }
}

// The number of a `@samples(N)`-style note, or 0 when there is none.
note_number :: (notes: [] string, prefix: string) -> string {
    for notes {
        if begins_with(it, prefix) && ends_with(it, ")") return string.{ it.count - prefix.count - 1, it.data + prefix.count };
    }

    return "0";
}

// The call that runs a `@property` test.
property_call :: (found: Found) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "__jacket_testing.run_property(t, %, %", found.name, ifx found.trials then found.trials else "__jacket_testing.DEFAULT_TRIALS");

    if found.generators.count {
        append(*builder, ", __jacket_testing.Custom_Generator.[");

        for found.generators {
            if it_index > 0 append(*builder, ", ");
            print_to_builder(*builder, "__jacket_testing.generator(%)", it);
        }

        append(*builder, "]");
    }

    append(*builder, ");");

    return builder_to_string(*builder,, temp);
}

// One field of a record: the bytes that frame records would split it.
field :: (text: string) -> string {
    copy := copy_string(text,, temp);
    for *copy {
        if it.* == 0x02 || it.* == 0x03 || it.* == 0x1f || it.* == #char "\n" || it.* == #char "\r" it.* = #char " ";
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

    source := registry_source(discovery);

    output: String_Builder;
    output.allocator = temp;

    for split(source, "\n") emit(*output, "source", it);

    for discovery.found {
        fields: [..] string;
        fields.allocator = temp;
        array_add(*fields, it.kind, it.name, it.file, tprint("%", it.line));
        for note: it.notes array_add(*fields, note);

        emit(*output, "manifest", ..fields);
    }

    // Last: a reader that waits for it finds everything else out.
    emit(*output, "discovered");
    print("%", builder_to_string(*output,, temp));
    add_build_string(source, discovery.workspace);
}

// Appends a record to `output`.
emit :: (output: *String_Builder, kind: string, fields: ..string) {
    append(output, "\x02");
    append(output, kind);

    for fields {
        append(output, "\x1f");
        append(output, field(it));
    }

    append(output, "\x03");
}

// The code that is added to the program: the test table and the entry point. The `@compile_time`
// tests run now, in the compiler (a `#run`), and the program replays what they reported.
registry_source :: (discovery: *Discovery) -> string {
    registry: String_Builder;

    append(*registry, "#scope_file\n__jacket_testing :: #import \"Extensions/Testing\";\n");

    // The compile-time tests: run now, in the compiler, and replayed by the program.
    compile_times := 0;
    for discovery.found if it.kind == "test" && it.compile_time compile_times += 1;

    if compile_times && discovery.list {
        append(*registry, "__jacket_ct_results :: \"\";\n");
    } else if compile_times {
        append(*registry, "__jacket_ct_results :: #run __jacket_testing.run_compile_time(__jacket_testing.Compile_Time_Case.[\n");

        for discovery.found {
            if it.kind != "test" || !it.compile_time continue;
            print_to_builder(*registry, "    .{ %, __jacket_ct_% },\n", quote(it.name), it_index);
        }

        append(*registry, "]);\n");
    }

    position := 0;

    for discovery.found {
        if it.kind != "test" continue;

        call := ifx it.property then property_call(it) else ifx it.cases then tprint("__jacket_testing.run_cases(t, %, %);", it.cases, it.name) else ifx it.arguments then tprint("%(t);", it.name) else tprint("%();", it.name);

        if it.compile_time {
            print_to_builder(*registry, "__jacket_ct_% :: (t: *__jacket_testing.Test) { % }\n", it_index, call);
            print_to_builder(*registry, "__jacket_test_% :: (t: *__jacket_testing.Test) { __jacket_testing.replay_compile_time(t, __jacket_ct_results, %); }\n", it_index, position);
            position += 1;
        } else {
            print_to_builder(*registry, "__jacket_test_% :: (t: *__jacket_testing.Test) { % }\n", it_index, call);
        }
    }

    append(*registry, "__jacket_tests :: __jacket_testing.Test_Case.[\n");

    for discovery.found {
        if it.kind != "test" continue;

        flags: [..] string;
        flags.allocator = temp;
        for note: it.notes if note == "allow_leaks" array_add(*flags, ".ALLOW_LEAKS");
        if it.cases array_add(*flags, ".CASES");

        print_to_builder(*registry, "    .{ %, __jacket_test_%, .{ %, %, 0 }, % },\n", quote(it.name), it_index, quote(it.file), it.line, ifx flags.count then join(..flags, separator = " | ") else "0");
    }

    append(*registry, "];\n");

    for discovery.found {
        if it.kind != "bench" continue;
        call := ifx it.arguments then tprint("%(b);", it.name) else tprint("%();", it.name);
        print_to_builder(*registry, "__jacket_bench_% :: (b: *__jacket_testing.Bench) { % }\n", it_index, call);
    }

    append(*registry, "__jacket_benches :: __jacket_testing.Bench_Case.[\n");

    for discovery.found {
        if it.kind != "bench" continue;
        print_to_builder(*registry, "    .{ %, __jacket_bench_%, %, % },\n", quote(it.name), it_index, note_number(it.notes, "samples("), note_number(it.notes, "min_time("));
    }

    append(*registry, "];\n#scope_export\n");
    print_to_builder(*registry, "% :: () { __jacket_testing.run_tests(__jacket_tests, __jacket_benches); }\n", ENTRY_POINT);

    return builder_to_string(*registry);
}

#import "Basic";
#import "Compiler";
#import "String";
#import "Sort";
