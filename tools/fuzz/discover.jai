// Metaprogram: lists the exported procedures of one stdlib module and prints a fuzzing harness.
// The driver (stdlib_fuzz.py) writes a wrapper that defines the constants below and #loads this file:
//   MODULE    the module name, e.g. "String"
//   NAMES     the module's top-level constant names, one per line (the driver reads them from its
//             module.jai: the compiler typechecks an export only once something refers to it)
//   DENY      procedure names to skip, one per line
//   ALLOW     when non-empty, the only procedure names to fuzz, one per line
//   POINTERS  procedure names that may receive pointers (null or a fresh allocation), one per line
//   NULLABLE  procedure names that also accept null pointers, one per line
//   ARITY     `name=N` lines: call `name` with only its first N arguments (the rest take defaults)
//   HOOKS     path of a file of `fuzz_*` procedures, or empty
// Output on stdout: `SKIP <name> <reason>` lines, then the harness between `HARNESS BEGIN` and
// `HARNESS END`.
#import "Basic";
#import "Compiler";
#import "Fuzz";

lines_of :: (text: string) -> [] string {
    result: [..] string;
    start := 0;
    for 0..text.count {
        if it == text.count || text[it] == 10 {
            line := slice_of(text, start, it - start);
            if line.count > 0 array_add(*result, line);
            start = it + 1;
        }
    }
    return result;
}

slice_of :: (s: string, from: s64, count: s64) -> string {
    result: string;
    result.data = s.data + from;
    result.count = count;
    return result;
}

listed :: (names: [] string, name: string) -> bool {
    for names if it == name return true;
    return false;
}

// Source text of a type, or an empty string when the harness cannot name or build it.
type_text :: (info: *Type_Info, exported: [] string) -> string {
    if info.type == {
        case .INTEGER;
            signed := (cast(*Type_Info_Integer) info).signed;
            if signed {
                if info.runtime_size == 1 return "s8";
                if info.runtime_size == 2 return "s16";
                if info.runtime_size == 4 return "s32";
                return "s64";
            }
            if info.runtime_size == 1 return "u8";
            if info.runtime_size == 2 return "u16";
            if info.runtime_size == 4 return "u32";
            return "u64";
        case .FLOAT;
            if info.runtime_size == 4 return "float32";
            return "float64";
        case .BOOL;   return "bool";
        case .STRING; return "string";
        case .ENUM;
            name := (cast(*Type_Info_Enum) info).name;
            if listed(exported, name) return tprint("Target.%", name);
            return "";
        case .STRUCT;
            s := cast(*Type_Info_Struct) info;
            if !listed(exported, s.name) return "";
            for m: s.members {
                if m.flags & .CONSTANT continue;
                if type_text(m.type, exported).count == 0 return "";
            }
            return tprint("Target.%", s.name);
        case .POINTER;
            inner := (cast(*Type_Info_Pointer) info).pointer_to;
            if inner.type == .VOID return "*void";
            text := type_text(inner, exported);
            if text.count == 0 return "";
            return tprint("*%", text);
        case .ARRAY;
            array := cast(*Type_Info_Array) info;
            if array.array_type == .RESIZABLE return "";
            text := type_text(array.element_type, exported);
            if text.count == 0 return "";
            if array.array_type == .VIEW return tprint("[] %", text);
            return tprint("[%] %", array.array_count, text);
    }
    return "";
}

Target_Proc :: struct {
    name: string;
    argument_types: [] string;
    pointers: bool;
    nullable: bool;
    arity: s64 = -1;
}

#run {
    deny := lines_of(DENY);
    allow := lines_of(ALLOW);
    pointer_ok := lines_of(POINTERS);
    arity := lines_of(ARITY);
    nullable := lines_of(NULLABLE);

    source: String_Builder;
    append(*source, "#import \"Basic\";\n#import \"Fuzz\";\n");
    append(*source, tprint("Target :: #import \"%\";\n", MODULE));
    if HOOKS.count > 0 append(*source, tprint("#load \"%\";\n", HOOKS));
    for lines_of(NAMES) append(*source, tprint("probe_% :: Target.%;\n", it, it));
    append(*source, "main :: () {}\n");

    w := compiler_create_workspace("fuzz-discovery");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(builder_to_string(*source), w);

    // `probe_<name> :: Target.<name>;` makes the compiler typecheck each export; the declaration's
    // type is the export's type. The same names tell which struct and enum types the harness can
    // spell as `Target.<name>`.
    exported := lines_of(NAMES);
    declared: [..] *Code_Declaration;
    hooks: [..] string;

    while true {
        m := compiler_wait_for_message();
        if m.kind == .COMPLETE break;
        if m.kind != .TYPECHECKED continue;
        tc := cast(*Message_Typechecked) m;
        for tc.declarations {
            d := it.expression;
            if starts_with_text(d.name, "probe_") array_add(*declared, d);
        }
        for tc.procedure_headers {
            h := it.expression;
            if starts_with_text(h.name, "fuzz_") && h.enclosing_load != null
                && h.enclosing_load.fully_pathed_filename == HOOKS {
                array_add(*hooks, h.name);
            }
        }
    }
    compiler_end_intercept(w);

    cases: [..] Target_Proc;
    for d: declared {
        if !(d.flags & .IS_CONSTANT) continue;
        name := slice_of(d.name, 6, d.name.count - 6);
        if d.type != null && d.type.type == .OVERLOAD_SET {
            if !listed(deny, name) print("SKIP % overload set\n", name);
            continue;
        }
        if d.type == null || d.type.type != .PROCEDURE continue;
        if listed(deny, name) continue;
        if allow.count > 0 && !listed(allow, name) continue;

        info := cast(*Type_Info_Procedure) d.type;
        if info.procedure_flags & .IS_POLYMORPHIC { print("SKIP % polymorphic\n", name); continue; }
        if info.procedure_flags & .IS_ELSEWHERE { print("SKIP % foreign\n", name); continue; }
        if info.procedure_flags & .IS_COMPILE_TIME_ONLY { print("SKIP % compile-time only\n", name); continue; }

        entry: Target_Proc;
        entry.name = name;
        entry.pointers = listed(pointer_ok, name);
        entry.nullable = listed(nullable, name);
        for arity {
            prefix := tprint("%=", name);
            if starts_with_text(it, prefix) {
                number, ok := to_integer(slice_of(it, prefix.count, it.count - prefix.count));
                if ok entry.arity = number;
            }
        }
        texts: [..] string;
        reason := "";
        for info.argument_types {
            if entry.arity >= 0 && it_index >= entry.arity break;
            text := type_text(it, exported);
            if text.count == 0 {
                reason = tprint("argument % has a type the generators cannot build", it_index);
                break;
            }
            if has_pointer(it) && !entry.pointers {
                reason = "takes a pointer and is not listed in POINTERS";
                break;
            }
            array_add(*texts, text);
        }
        if reason.count > 0 { print("SKIP % %\n", name, reason); continue; }
        entry.argument_types = texts;
        array_add(*cases, entry);
    }

    out: String_Builder;
    append(*out, "#import \"Basic\";\n#import \"Fuzz\";\n");
    append(*out, tprint("Target :: #import \"%\";\n", MODULE));
    if HOOKS.count > 0 append(*out, tprint("#load \"%\";\n", HOOKS));
    for cases {
        append(*out, tprint("thunk_% :: (r: *Rng) {\n", it_index));
        arguments: String_Builder;
        for it.argument_types {
            append(*out, tprint("    a% := gen(r, %);\n", it_index, it));
            if it_index > 0 append(*arguments, ", ");
            append(*arguments, tprint("a%", it_index));
        }
        append(*out, tprint("    Target.%(%);\n}\n", it.name, builder_to_string(*arguments)));
    }
    for hooks append(*out, tprint("hook_% :: (r: *Rng) { % (r); }\n", it_index, it));
    append(*out, "CASES :: Case.[\n");
    for cases append(*out, tprint("    .{\"%.%\", thunk_%, %, %},\n", MODULE, it.name, it_index, it.pointers, it.nullable));
    for hooks append(*out, tprint("    .{\"%.%\", hook_%, true, false},\n", MODULE, it, it_index));
    append(*out, "];\nmain :: () {\n    #if ENGINE_BUILD exit(run_engine(CASES));\n    else exit(run_cases(CASES));\n}\n");

    print("HARNESS BEGIN\n%HARNESS END\n", builder_to_string(*out));
}

starts_with_text :: (s: string, prefix: string) -> bool {
    if s.count < prefix.count return false;
    return slice_of(s, 0, prefix.count) == prefix;
}

ends_with_text :: (s: string, suffix: string) -> bool {
    if s.count < suffix.count return false;
    return slice_of(s, s.count - suffix.count, suffix.count) == suffix;
}
