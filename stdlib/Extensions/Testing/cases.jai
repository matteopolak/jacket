// Tests with cases: `@cases(LIST)` runs a test once per entry of a list, each entry a subtest
// named from its values. See docs/extensions/testing.md.

// Runs `body` once per entry of `cases`, each as the subtest `test/<values>`; returns whether
// every case passed. An entry is the argument of `body`, or, when `body` takes several, a struct
// whose members are its arguments in order. `body` may take a leading `*Test`. This is what a
// `@test` procedure with a `@cases(LIST)` note runs; call it by hand to build the list at run
// time or to run a helper per case:
//
// ```jai
// Case :: struct { input: string; length: s64; }
//
// checks_each_case :: (t: *Test) {
//     lengths :: (input: string, length: s64) {
//         expect_equal(input.count, length);
//     }
//
//     passed := run_cases(t, Case.[.{ "", 0 }, .{ "abc", 3 }], lengths);
//     expect(passed);
// }
// ```
//
// - t: the running test.
// - cases: the entries to run, one subtest each.
// - $body: the procedure each entry is passed to.
run_cases :: (t: *Test, cases: [] $T, $body: $B) -> bool {
    names := case_names(cases, body);

    if listing_cases {
        list_case_names(t, names);

        return true;
    }

    passed := true;

    for cases {
        entry := it;

        ok := run(t, names[it_index], (t: *Test, c: *T) {
            #insert -> string {
                return call_source(T, B);
            }
        }, *entry);

        if !ok passed = false;
    }

    return passed;
}

// Every member of the enum `T`, in declaration order, as a list for `@cases`:
//
// ```jai
// Color :: enum { RED; GREEN; BLUE; }
//
// values := all_values(Color);
//
// assert(values.count == 3);
// assert(values[0] == .RED && values[2] == .BLUE);
// ```
//
// Pass it to a test as `@cases(all_values(Color))`. The array is in temporary storage, so it lasts for the test that asked for it.
//
// - $T: the enum type.
all_values :: ($T: Type) -> [] T #modify {
    return (cast(*Type_Info) T).type == .ENUM;
} {
    info := cast(*Type_Info_Enum) T;
    values: [..] T;
    values.allocator = temp;

    for info.values array_add(*values, cast,no_check(T) it);

    return values;
}

// The names `run_cases` gives the subtests of `cases`: the entry's values, without `/`, shortened,
// repeats numbered. In temporary storage.
//
// - cases: the entries to name.
// - body: the procedure they are passed to.
case_names :: (cases: [] $T, body: $B) -> [] string {
    names: [..] string;
    names.allocator = temp;

    for *cases {
        index := it_index;
        name := tidy(case_label(it, body), index);

        for other: names {
            if other == name {
                name = tprint("% #%", name, index);
                break;
            }
        }

        array_add(*names, name);
    }

    return names;
}

#scope_file

MAX_NAME :: 60;

case_label :: (c: *$T, body: $B) -> string {
    #insert -> string {
        return label_source(T, B);
    }
}

// One value of an entry as text: strings as they are (`""` when empty), the rest as `print` shows them.
append_label :: (builder: *String_Builder, value: $V) {
    #if V == string {
        append(builder, ifx value then value else "\"\"");
    } else {
        print_to_builder(builder, "%", value);
    }
}

label_of :: (value: $V) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append_label(*builder, value);

    return builder_to_string(*builder,, temp);
}

// The values of the entry, as the name of its subtest.
label_source :: (T: Type, B: Type) -> string {
    values := value_parameters(B);
    if values == 1 return "return label_of(c.*);";

    members := argument_members(T, values);
    if !members.count return "#assert false \"@cases: a test with several arguments takes entries that are structs with one member per argument\";";

    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "builder: String_Builder;\nbuilder.allocator = temp;\n");

    for members {
        if it_index > 0 append(*builder, "append(*builder, \",\");\n");
        append(*builder, "append_label(*builder, c.");
        append(*builder, it);
        append(*builder, ");\n");
    }

    append(*builder, "return builder_to_string(*builder,, temp);");

    return builder_to_string(*builder,, temp);
}

// The source of the call of `body` with the entry `c` (a `*T`) and the subtest `t`.
call_source :: (T: Type, B: Type) -> string {
    info := cast(*Type_Info_Procedure) B;
    values := value_parameters(B);
    leading := info.argument_types.count - values;
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "body(");
    if leading append(*builder, "t");

    if values == 0 {
        return "#assert false \"@cases: the test takes no argument to pass an entry to\";";
    } else if values == 1 {
        append(*builder, ifx leading then ", c.*" else "c.*");
    } else {
        members := argument_members(T, values);
        if !members.count return "#assert false \"@cases: a test with several arguments takes entries that are structs with one member per argument\";";

        for members {
            if it_index > 0 || leading append(*builder, ", ");
            print_to_builder(*builder, "c.%", it);
        }
    }

    append(*builder, ");");

    return builder_to_string(*builder,, temp);
}

// How many arguments of the procedure type `B` are not its leading `*Test`.
value_parameters :: (B: Type) -> s64 {
    info := cast(*Type_Info_Procedure) B;
    count := info.argument_types.count;
    if count > 0 && info.argument_types[0] == type_info(*Test) count -= 1;

    return count;
}

// The names of the members of the struct `T` when there are `count` of them, else none.
argument_members :: (T: Type, count: s64) -> [] string {
    info := cast(*Type_Info) T;
    if info.type != .STRUCT return .[];

    names: [..] string;
    names.allocator = temp;

    for (cast(*Type_Info_Struct) info).members {
        if !(it.flags & .CONSTANT) array_add(*names, it.name);
    }

    if names.count != count return .[];

    return names;
}

// A subtest name for `label`: no `/` (it separates a test from its subtest), no control
// characters, shortened, and `case N` when nothing is left.
tidy :: (label: string, index: s64) -> string {
    if !label return tprint("case %", index);

    limit := min(label.count, MAX_NAME);
    // Not in the middle of a UTF-8 sequence.
    while limit < label.count && limit > 0 && (label[limit] & 0xC0) == 0x80 limit -= 1;

    name := copy_string(string.{ limit, label.data },, temp);

    for *name {
        if it.* == #char "/" it.* = #char "|";
        else if it.* < 0x20 || it.* == 0x7F it.* = #char " ";
    }

    if limit < label.count name = tprint("%...", name);

    return name;
}
