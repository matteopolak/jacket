// Snapshots: `expect_snapshot(t, value)` compares the printed form of a value with a file kept
// next to the test. See docs/extensions/testing.md.
//
// The file of a snapshot is `__snapshots__/<test file>/<test>[/<case>].snap` in the folder of
// the test's file. A snapshot that differs, or does not exist, fails the test and is written
// as `.snap.new` beside it; `jacket test --accept` writes the new snapshots in place and
// `jacket test --review` goes through them one by one.

// Compares the printed form of `value` with its stored snapshot. Strings are compared as they
// are; any other value is printed with one member or element to a line, so a change shows as
// a line diff. A test that takes several snapshots numbers them, or names them with `name`.
//
// ```jai,no_run
// Point :: struct { x, y: s32; }
//
// prints_a_point :: (t: *Test) {
//     expect_snapshot(t, Point.{ 1, 2 });
//     expect_snapshot(t, "a block of text\nwith two lines", "text");
// } @test
// ```
expect_snapshot :: (t: *Test, value: Any, name := "", loc := #caller_location) -> bool {
    return check_snapshot(t, snapshot_text(value), name, loc);
}

expect_snapshot :: (t: *Test, text: string, name := "", loc := #caller_location) -> bool {
    return check_snapshot(t, text, name, loc);
}

// `value` as the text a snapshot of it holds.
snapshot_text :: (value: Any) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    inner := underlying(value);

    if inner.type.type == .STRING {
        append(*builder, (cast(*string) inner.value_pointer).*);
    } else {
        write_value(*builder, inner, 0);
    }

    return builder_to_string(*builder,, temp);
}

#scope_module

// Set by the test program's `--accept`: new snapshots replace the stored ones.
accepting := false;

#scope_file

SNAPSHOTS_FOLDER :: "__snapshots__";
WIDE :: 72;

check_snapshot :: (t: *Test, text: string, name: string, loc: Source_Code_Location) -> bool {
    path := snapshot_path(t, name, loc);
    got := normalized(text);

    if !make_folder_for(path) {
        report_failure(loc, tprint("cannot create the folder of the snapshot %", path));
        return false;
    }

    pending := tprint("%.new", path);
    stored, found := read_entire_file(path, log_errors = false,, temp);
    want := normalized(stored);

    if found && got == want {
        // A snapshot that holds no longer needs the one that was waiting to be reviewed.
        file_delete(pending);
        return true;
    }

    if accepting {
        if write_entire_file(path, tprint("%\n", got)) {
            file_delete(pending);
            return true;
        }

        report_failure(loc, tprint("cannot write the snapshot %", path));

        return false;
    }

    if !write_entire_file(pending, tprint("%\n", got)) {
        report_failure(loc, tprint("cannot write the snapshot %", pending));
        return false;
    }

    if found {
        report_failure(loc, tprint("the snapshot % differs\n%\nthe new one is in %.new; `jacket test --accept` keeps it, `jacket test --review` goes through them", shown(path), describe_difference(got, want), shown(path)));
    } else {
        report_failure(loc, tprint("there is no snapshot % yet\nthe new one is in %.new; `jacket test --accept` keeps it, `jacket test --review` goes through them", shown(path), shown(path)));
    }

    return false;
}

// The text without carriage returns and without line breaks at the end, so a snapshot checked
// out with another OS's line endings still matches.
normalized :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for text if it != #char "\r" append(*builder, it);

    result := builder_to_string(*builder,, temp);
    while result.count > 0 && result[result.count - 1] == #char "\n" result.count -= 1;

    return result;
}

// `<folder of the test file>/__snapshots__/<file name>/<test>[/<case>][.<name or number>].snap`
snapshot_path :: (t: *Test, name: string, loc: Source_Code_Location) -> string {
    file := running_location().fully_pathed_filename;
    if !file file = loc.fully_pathed_filename;

    cut := last_separator(file);
    folder := ifx cut >= 0 then string.{ cut, file.data } else ".";
    base := ifx cut >= 0 then string.{ file.count - cut - 1, file.data + cut + 1 } else file;
    dot := last_dot(base);
    stem := ifx dot > 0 then string.{ dot, base.data } else base;

    builder: String_Builder;
    builder.allocator = temp;
    print_to_builder(*builder, "%/%/%", folder, SNAPSHOTS_FOLDER, safe_name(stem));

    start := 0;

    for t.name {
        if it != #char "/" continue;
        print_to_builder(*builder, "/%", safe_name(string.{ it_index - start, t.name.data + start }));
        start = it_index + 1;
    }

    print_to_builder(*builder, "/%", safe_name(string.{ t.name.count - start, t.name.data + start }));

    t.snapshots += 1;

    if name {
        print_to_builder(*builder, ".%", safe_name(name));
    } else if t.snapshots > 1 {
        print_to_builder(*builder, ".%", t.snapshots);
    }

    append(*builder, ".snap");

    return builder_to_string(*builder,, temp);
}

last_separator :: (path: string) -> s64 {
    i := path.count - 1;
    while i >= 0 {
        if path[i] == #char "/" || path[i] == #char "\\" return i;
        i -= 1;
    }

    return -1;
}

last_dot :: (name: string) -> s64 {
    i := name.count - 1;
    while i >= 0 {
        if name[i] == #char "." return i;
        i -= 1;
    }

    return -1;
}

// A name that is a legal file name on every OS: other characters become `_`, and a name that
// changed gets a hash of the original so two different names stay apart. Case names are the
// values of an entry, so they may hold anything.
safe_name :: (name: string) -> string {
    if !name return "_";

    builder: String_Builder;
    builder.allocator = temp;
    changed := false;

    for name {
        keep := (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || (it >= #char "0" && it <= #char "9") || it == #char "_" || it == #char "-" || it == #char "," || it == #char "=" || it == #char "+" || it == #char "#" || it == #char "." || (it == #char " " && it_index > 0 && it_index + 1 < name.count);
        if keep {
            append(*builder, it);
        } else {
            append(*builder, "_");
            changed = true;
        }
    }

    // Windows drops a trailing dot and does not allow the names of its devices.
    result := builder_to_string(*builder,, temp);
    if result[0] == #char "." || result[result.count - 1] == #char "." changed = true;

    if !changed return result;

    hash: u64 = 0xcbf29ce484222325;

    for name {
        hash = (hash ^ it) * 0x100000001b3;
    }

    return tprint("%~%", trimmed_dots(result), formatInt(hash & 0xffffff, base = 16, minimum_digits = 6));
}

trimmed_dots :: (name: string) -> string {
    result := name;
    while result.count > 0 && result[result.count - 1] == #char "." result.count -= 1;

    if result.count > 0 && result[0] == #char "." {
        result.data += 1;
        result.count -= 1;
    }

    return ifx result then result else "_";
}

// Creates the folders above `path`.
make_folder_for :: (path: string) -> bool {
    cut := last_separator(path);
    if cut <= 0 return true;

    return make_directory_if_it_does_not_exist(string.{ cut, path.data }, recursive = true);
}

// A path as messages show it: from the last folder above `__snapshots__` (`tests/__snapshots__/...`),
// since the whole path says nothing the reader does not know.
shown :: (path: string) -> string {
    marker := tprint("/%/", SNAPSHOTS_FOLDER);

    for 0..path.count - marker.count {
        if string.{ marker.count, path.data + it } != marker continue;

        start := it;
        while start > 0 && path[start - 1] != #char "/" && path[start - 1] != #char "\\" start -= 1;

        return string.{ path.count - start, path.data + start };
    }

    return path;
}

// One value of any type as lines: a struct member or an array element to a line, indented
// two spaces per level, so a change is a one-line diff.
write_value :: (builder: *String_Builder, value: Any, depth: s64) {
    info := value.type;

    if info.type == .VARIANT {
        write_value(builder, underlying(value), depth);
        return;
    }

    if info.type == {
        case .STRING;
            append(builder, quoted((cast(*string) value.value_pointer).*));
            return;
        case .STRUCT;
            write_struct(builder, value, depth);
            return;
        case .ARRAY;
            write_array(builder, value, depth);
            return;
        case .POINTER; #through;
        case .PROCEDURE;
            append(builder, ifx (cast(**void) value.value_pointer).* then "<pointer>" else "null");
            return;
    }

    print_to_builder(builder, "%", value);
}

write_indent :: (builder: *String_Builder, depth: s64) {
    for 1..depth append(builder, "  ");
}

write_struct :: (builder: *String_Builder, value: Any, depth: s64) {
    info := cast(*Type_Info_Struct) value.type;

    print_type_to_builder(builder, value.type);
    append(builder, " {\n");

    for info.members {
        if it.flags & .CONSTANT continue;

        write_indent(builder, depth + 1);
        print_to_builder(builder, "% = ", it.name);
        write_value(builder, Any.{ it.type, value.value_pointer + it.offset_in_bytes }, depth + 1);
        append(builder, "\n");
    }

    write_indent(builder, depth);
    append(builder, "}");
}

write_array :: (builder: *String_Builder, value: Any, depth: s64) {
    element_type, count, data := array_view(value);

    if count == 0 {
        append(builder, "[]");
        return;
    }

    // A short list of plain values stays on one line.
    if is_plain(element_type) {
        line: String_Builder;
        line.allocator = temp;
        append(*line, "[");

        for 0..count - 1 {
            if it > 0 append(*line, ", ");
            write_value(*line, element(element_type, data, it), depth + 1);
        }

        append(*line, "]");
        text := builder_to_string(*line,, temp);

        if text.count <= WIDE {
            append(builder, text);
            return;
        }
    }

    append(builder, "[\n");

    for 0..count - 1 {
        write_indent(builder, depth + 1);
        write_value(builder, element(element_type, data, it), depth + 1);
        append(builder, ",\n");
    }

    write_indent(builder, depth);
    append(builder, "]");
}

is_plain :: (info: *Type_Info) -> bool {
    return info.type != .STRUCT && info.type != .ARRAY && info.type != .VARIANT;
}
