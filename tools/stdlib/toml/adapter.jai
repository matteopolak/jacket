// toml-test tagged-JSON decoder and encoder. The first argument selects encode; default decode.
#import "Basic";
#import "String";
T :: #import "Extensions/Toml";
J :: #import "Extensions/Json";
Env :: #import "Extensions/Environment";

#if OS == .WASM {
    libc :: #system_library "libc";
    stdin_read :: (fd: s32, buffer: *void, count: u64) -> s64 #foreign libc "read";
} else {
    F :: #import "File";

    #if OS == .WINDOWS {
        W :: #import "Windows";
    } else {
        P :: #import "POSIX";
    }
}

// Reads the protocol stream without introducing native filesystem dependencies on WASI.
read_chunk :: (buffer: *u8, capacity: int) -> bool, int {
    #if OS == .WASM {
        count := stdin_read(0, buffer, cast(u64) capacity);
        return count >= 0, count;
    } else {
        input: F.File;

        #if OS == .WINDOWS {
            input = .{ W.GetStdHandle(-10) };
        } else {
            input = .{ P.stdin };
        }

        ok, count := F.file_read(input, buffer, capacity);

        return ok, count;
    }
}

main :: () {
    bytes: [..] u8;

    defer array_free(bytes);
    buffer: [4096] u8;

    while true {
        ok, count := read_chunk(buffer.data, buffer.count);
        if !ok exit(74);
        if !count break;
        for 0..count - 1 array_add(*bytes, buffer[it]);
        if bytes.count > 64 << 20 exit(1);
    }

    text := string.{ bytes.count, bytes.data };
    args := get_command_line_arguments();
    encoding := (args.count > 1 && args[1] == "encode") || (args.count && (ends_with(args[0], "encoder") || ends_with(args[0], "encoder.exe")));

    if encoding {
        tagged, json_error := J.from_json(J.Json_Value, text);
        if json_error.kind != .NONE exit(1);

        defer J.free_json(*tagged);
        doc, document_error := T.new_toml_document();
        if document_error.kind != .NONE exit(1);

        defer T.free_toml(*doc);
        if tagged.kind != .OBJECT exit(1);

        for *tagged.members {
            node, ok := untag(*doc, *it.value);
            if !ok exit(1);
            error := T.toml_insert(*doc, doc.root, it.key, node);
            if error.kind != .NONE exit(1);
        }

        output, output_error := T.write_toml(doc.root);
        if output_error.kind != .NONE exit(1);

        defer free(output);
        write_string(output);
    } else {
        doc, error := T.parse_toml(text, .{ record_statements = Env.environment_variable("TOML_CONFORMANCE_STATEMENTS") == "1" });
        if error.kind != .NONE exit(1);

        defer T.free_toml(*doc);
        value := tag(doc.root);

        defer J.free_json(*value);
        output := J.to_json(value);

        defer free(output);
        write_string(output);
    }
}

tag :: (node: *T.Toml_Value) -> J.Json_Value {
    if node.kind == .TABLE {
        out := J.json_object();
        for node.members J.json_set(*out, it.key, tag(it.value));
        return out;
    }

    if node.kind == .ARRAY {
        out := J.json_array();
        for node.items J.json_push(*out, tag(it));
        return out;
    }

    out := J.json_object();
    kind: string;
    value: string;

    if node.kind == .STRING {
        kind = "string";
        value = node.text;
    } else if node.kind == .BOOL {
        kind = "bool";
        value = ifx node.boolean then "true" else "false";
    } else if node.kind == .INTEGER {
        kind = "integer";
        value = tprint("%", node.integer);
    } else {
        if node.kind == .FLOAT kind = "float";
        if node.kind == .LOCAL_DATE kind = "date-local";
        if node.kind == .LOCAL_TIME kind = "time-local";
        if node.kind == .LOCAL_DATETIME kind = "datetime-local";
        if node.kind == .OFFSET_DATETIME kind = "datetime";

        member := T.Toml_Member.{ key = "value", value = node };
        root: T.Toml_Value;
        root.kind = .TABLE;
        array_add(*root.members, member);
        output, error := T.write_toml(*root);

        defer free(output);
        array_free(root.members);
        assert(error.kind == .NONE);
        equal := find_index_from_left(output, #char "=");

        value = trim(slice(output, equal + 1, output.count - equal - 1));
        J.json_set(*out, "type", J.json_string(kind));
        J.json_set(*out, "value", J.json_string(value));

        return out;
    }

    J.json_set(*out, "type", J.json_string(kind));
    J.json_set(*out, "value", J.json_string(value));

    return out;
}

// Construct semantic nodes through the public owning builder. Floats stay floats even when
// the tagged JSON spells an integral value without a decimal point.
untag :: (doc: *T.Toml_Document, value: *J.Json_Value) -> *T.Toml_Value, bool {
    if value.kind == .ARRAY {
        node, error := T.toml_new_value(doc, .ARRAY);
        if error.kind != .NONE return null, false;

        for *value.items {
            child, ok := untag(doc, it);
            if !ok return null, false;
            error = T.toml_push(doc, node, child);
            if error.kind != .NONE return null, false;
        }

        return node, true;
    }

    if value.kind != .OBJECT return null, false;

    kind := J.json_get(value, "type");
    scalar := J.json_get(value, "value");

    if kind && scalar && kind.kind == .STRING && scalar.kind == .STRING && value.members.count == 2 {
        node_kind: T.Toml_Kind;
        if kind.text == "string" node_kind = .STRING;
        else if kind.text == "integer" node_kind = .INTEGER;
        else if kind.text == "float" node_kind = .FLOAT;
        else if kind.text == "bool" node_kind = .BOOL;
        else if kind.text == "datetime" node_kind = .OFFSET_DATETIME;
        else if kind.text == "datetime-local" node_kind = .LOCAL_DATETIME;
        else if kind.text == "date-local" node_kind = .LOCAL_DATE;
        else if kind.text == "time-local" node_kind = .LOCAL_TIME;
        else return null, false;

        node, error := T.toml_new_value(doc, node_kind);
        if error.kind != .NONE return null, false;

        if node_kind == .STRING {
            error = T.toml_set_text(doc, node, scalar.text);
            if error.kind != .NONE return null, false;
        } else if node_kind == .INTEGER {
            number: J.Json_Value;
            number.kind = .NUMBER;
            number.text = scalar.text;
            integer, ok := J.json_as_s64(*number);
            if !ok return null, false;

            node.integer = integer;
        } else if node_kind == .FLOAT {
            text := scalar.text;
            negative := text.count && text[0] == 45;
            word := text;

            if text.count && (text[0] == 45 || text[0] == 43) word = slice(text, 1, text.count - 1);

            if word == "nan" || word == "inf" {
                bits: u64 = ifx word == "nan" then 0x7ff8000000000000 else 0x7ff0000000000000;
                if negative bits |= 0x8000000000000000;
                node.real = cast,force(float64) bits;
            } else {
                real, ok, remainder := string_to_float64(text);
                if !ok || remainder.count return null, false;
                node.real = real;
            }
        } else if node_kind == .BOOL {
            if scalar.text != "true" && scalar.text != "false" return null, false;
            node.boolean = scalar.text == "true";
        } else {
            parsed, parse_error := T.parse_toml(tprint("value = %", scalar.text));
            if parse_error.kind != .NONE return null, false;

            defer T.free_toml(*parsed);
            temporal := T.toml_get(parsed.root, "value");
            if !temporal || temporal.kind != node_kind return null, false;

            node.temporal = temporal.temporal;
            node.temporal.fraction = "";
            error = T.toml_set_fraction(doc, node, temporal.temporal.fraction);
            if error.kind != .NONE && node_kind != .LOCAL_DATE return null, false;
        }

        return node, true;
    }

    table, error := T.toml_new_value(doc, .TABLE);
    if error.kind != .NONE return null, false;

    for *value.members {
        child, ok := untag(doc, *it.value);
        if !ok return null, false;
        error = T.toml_insert(doc, table, it.key, child);
        if error.kind != .NONE return null, false;
    }

    return table, true;
}
