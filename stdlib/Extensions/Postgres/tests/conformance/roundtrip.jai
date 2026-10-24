// The binary round trip: every column of every type the client decodes, fetched in text and in
// binary, decoded with the client's codecs, compared, then sent back as a parameter and compared
// on the server. See docs/postgres/conformance.md.
//
//     jaic roundtrip.jai -exe roundtrip
//     PGHOST=/tmp/pgsock PGPORT=5432 PGUSER=postgres ./roundtrip --database regression \
//         --schema '*' --script edge_values.sql --json-literals literals.txt
//
// A value passes when:
// 1. the text and the binary result decode to the same Jai value;
// 2. the server finds the decoded value, sent back as a parameter, equal to the original (`is not
//    distinct from`) and printing the same text (so -0.0 and a numeric's scale count);
// 3. for json and jsonb: the document decoded into a Json_Value and written back by Pg.json is
//    the same document on the server.
//
// Arrays compare through to_jsonb, because a Jai slice has no lower bound: '[0:1]={1,2}' comes
// back as '{1,2}', and its text is only compared when the original had no bounds.

#import "Basic";
#import "String";
#import "File";
#import "Hash_Table";
#import "Extensions/Environment";
Io :: #import "Extensions/Io";
Pg :: #import "Extensions/Postgres";
Json :: #import "Extensions/Json";

MAX_PRINTED_FAILURES :: 60;

Type_Stats :: struct {
    name: string;
    values: s64; // non-null values fetched
    distinct: s64; // distinct values checked and sent back
    nulls: s64;
    skipped: s64; // values the client does not decode (multi-dimensional arrays)
    failures: s64;
}

// One column being checked.
Check :: struct {
    db: *Pg.Connection;
    label: string; // schema.table.column
    select_sql: string;
    sql_type: string; // for casts: "int4", "text[]"
    exact_text: bool; // compare the server's text of the value sent back
}

all_stats: [..] Type_Stats;
seen: Table(string, bool);
total_failures: s64;
unsupported_columns: [..] string;
rejected_literals: s64;
json_without_jsonb: s64;
unpaired_surrogates: s64;

exit_code: s64;

main :: () {
    Io.io_run(() {
        exit_code = run(get_command_line_arguments());
    });

    exit(cast(s32) exit_code);
}

run :: (args: [] string) -> s64 {
    database := environment_variable("PGDATABASE");
    script, literals: string;
    schemas: [..] string;

    i := 1;

    while i < args.count {
        flag := args[i];

        if i + 1 >= args.count {
            print("% needs a value\n", flag);
            return 2;
        }

        value := args[i + 1];

        i += 2;

        if flag == "--database" database = value;
        else if flag == "--schema" array_add(*schemas, value);
        else if flag == "--script" script = value;
        else if flag == "--json-literals" literals = value;
        else {
            print("unknown option %\n", flag);
            return 2;
        }
    }

    if !schemas.count array_add(*schemas, "public");

    config := config_from_environment(database);
    db, error := Pg.connect(config);

    if error.kind != .NONE {
        print("cannot connect: %\n", Pg.error_text(error));
        return 2;
    }

    defer Pg.close(db);

    if script {
        sql, ok := read_entire_file(script, log_errors = false);

        if !ok {
            print("cannot read %\n", script);
            return 2;
        }

        error = Pg.execute_script(db, sql);

        if error.kind != .NONE {
            print("% failed: %\n", script, Pg.error_text(error));
            return 2;
        }
    }

    check_schemas(db, schemas);
    if literals check_literals(db, literals);

    report();

    return ifx total_failures then 1 else 0;
}

config_from_environment :: (database: string) -> Pg.Config {
    config: Pg.Config;
    config.ssl_mode = .DISABLE;
    config.host = "localhost";
    config.json_options.duplicate_keys = .LAST;

    if environment_variable("PGHOST") config.host = environment_variable("PGHOST");

    if environment_variable("PGPORT") {
        port, valid := string_to_int(environment_variable("PGPORT"));
        if valid config.port = cast(u16) port;
    }

    config.user = ifx environment_variable("PGUSER") then environment_variable("PGUSER") else "postgres";
    config.database = ifx database then database else "postgres";

    // Text timestamps carry the zone's offset, including the odd offsets of old local mean time.
    zone := ifx environment_variable("ROUNDTRIP_TIMEZONE") then environment_variable("ROUNDTRIP_TIMEZONE") else "Europe/Amsterdam";

    config.parameters = Pg.Startup_Parameter.[.{ "TimeZone", zone }];

    return config;
}

// Every column of the ordinary, partitioned and materialized tables in these schemas ("*": all
// but the system's).
check_schemas :: (db: *Pg.Connection, schemas: [] string) {
    sql := #string SQL
        select n.nspname::text, c.relname::text, a.attname::text
        from pg_attribute a
        join pg_class c on c.oid = a.attrelid
        join pg_namespace n on n.oid = c.relnamespace
        where ('*' = any($1) or n.nspname::text = any($1)) and n.nspname !~ '^pg_' and n.nspname <> 'information_schema'
          and c.relkind in ('r', 'p', 'm') and a.attnum > 0 and not a.attisdropped
        order by 1, 2, a.attnum
    SQL;

    columns, error := Pg.query_dynamic(db, sql, schemas);

    if error.kind != .NONE {
        print("cannot list columns: %\n", Pg.error_text(error));
        total_failures += 1;
        return;
    }

    defer Pg.free_result(*columns);

    for row: 0..columns.row_count - 1 {
        auto_release_temp();

        schema := text_of(*columns, row, 0);
        table := text_of(*columns, row, 1);
        column := text_of(*columns, row, 2);

        check_column(db, schema, table, column);
    }
}

text_of :: (result: *Pg.Result, row: s64, column: s64) -> string {
    value, error := Pg.get(result, row, column, string);
    assert(error.kind == .NONE);

    return copy_string(value,, temp);
}

quote_identifier :: (name: string) -> string {
    return tprint("\"%\"", replace(name, "\"", "\"\"",, temp));
}

check_column :: (db: *Pg.Connection, schema: string, table: string, column: string) {
    check: Check;
    check.db = db;
    check.label = tprint("%.%.%", schema, table, column);
    check.select_sql = tprint("select % from %.%", quote_identifier(column), quote_identifier(schema), quote_identifier(table));
    check.exact_text = true;

    // A domain reports its base type here.
    probe, error := Pg.query_binary(db, tprint("% limit 0", check.select_sql));

    if error.kind != .NONE {
        print("cannot read %: %\n", check.label, Pg.error_text(error));
        total_failures += 1;
        Pg.free_error(*error);
        return;
    }

    oid := probe.columns[0].type_oid;

    Pg.free_result(*probe);

    if !check_oid(*check, cast(Pg.Oid) oid) array_add(*unsupported_columns, sprint("% (type %)", check.label, oid));
}

// Picks the Jai type for a column type; false when the client does not decode it.
check_oid :: (check: *Check, oid: Pg.Oid) -> bool {
    if oid == {
        case .BOOL; check_type(check, bool, "bool");
        case .BYTEA; check_type(check, [] u8, "bytea");
        case .CHAR; check_type(check, string, "\"char\"");
        case .NAME; check_type(check, string, "name");
        case .INT8; check_type(check, s64, "int8");
        case .INT2; check_type(check, s16, "int2");
        case .INT4; check_type(check, s32, "int4");
        case .TEXT; check_type(check, string, "text");
        case .OID; check_type(check, u32, "oid");
        case .FLOAT4; check_type(check, float32, "float4");
        case .FLOAT8; check_type(check, float64, "float8");
        case .BPCHAR; check_type(check, string, "bpchar");
        case .VARCHAR; check_type(check, string, "varchar");
        case .DATE; check_type(check, Pg.Date, "date");
        case .TIME; check_type(check, Pg.Time, "time");
        case .TIMESTAMP; check_type(check, Pg.Local_Timestamp, "timestamp");
        case .TIMESTAMPTZ; check_type(check, Pg.Timestamp, "timestamptz");
        case .INTERVAL; check_type(check, Pg.Interval, "interval");
        case .NUMERIC; check_type(check, Pg.Numeric, "numeric");
        case .UUID; check_type(check, Pg.Uuid, "uuid");
        case .JSON; check_json_column(check, "json");
        case .JSONB; check_json_column(check, "jsonb");

        case .BOOL_ARRAY; check_type(check, [] Pg.Nullable(bool), "bool[]");
        case .BYTEA_ARRAY; check_type(check, [] Pg.Nullable([] u8), "bytea[]");
        case .CHAR_ARRAY; check_type(check, [] Pg.Nullable(string), "\"char\"[]");
        case .NAME_ARRAY; check_type(check, [] Pg.Nullable(string), "name[]");
        case .INT2_ARRAY; check_type(check, [] Pg.Nullable(s16), "int2[]");
        case .INT4_ARRAY; check_type(check, [] Pg.Nullable(s32), "int4[]");
        case .TEXT_ARRAY; check_type(check, [] Pg.Nullable(string), "text[]");
        case .BPCHAR_ARRAY; check_type(check, [] Pg.Nullable(string), "bpchar[]");
        case .VARCHAR_ARRAY; check_type(check, [] Pg.Nullable(string), "varchar[]");
        case .INT8_ARRAY; check_type(check, [] Pg.Nullable(s64), "int8[]");
        case .OID_ARRAY; check_type(check, [] Pg.Nullable(u32), "oid[]");
        case .FLOAT4_ARRAY; check_type(check, [] Pg.Nullable(float32), "float4[]");
        case .FLOAT8_ARRAY; check_type(check, [] Pg.Nullable(float64), "float8[]");
        case .TIMESTAMP_ARRAY; check_type(check, [] Pg.Nullable(Pg.Local_Timestamp), "timestamp[]");
        case .DATE_ARRAY; check_type(check, [] Pg.Nullable(Pg.Date), "date[]");
        case .TIME_ARRAY; check_type(check, [] Pg.Nullable(Pg.Time), "time[]");
        case .TIMESTAMPTZ_ARRAY; check_type(check, [] Pg.Nullable(Pg.Timestamp), "timestamptz[]");
        case .INTERVAL_ARRAY; check_type(check, [] Pg.Nullable(Pg.Interval), "interval[]");
        case .NUMERIC_ARRAY; check_type(check, [] Pg.Nullable(Pg.Numeric), "numeric[]");
        case .UUID_ARRAY; check_type(check, [] Pg.Nullable(Pg.Uuid), "uuid[]");
        // json arrays keep each element's text, which the cast from jsonb[] does not.
        case .JSON_ARRAY;
            check.exact_text = false;
            check_type(check, [] Pg.Nullable(Pg.Json_Raw), "json[]");
        case .JSONB_ARRAY; check_type(check, [] Pg.Nullable(Pg.Json_Raw), "jsonb[]");

        case; return false;
    }

    return true;
}

stats_for :: (name: string) -> *Type_Stats {
    for *all_stats if it.name == name return it;

    array_add(*all_stats, .{ name = copy_string(name) });

    return *all_stats[all_stats.count - 1];
}

fail :: (check: *Check, row: s64, original: string, message: string) {
    stats_for(check.sql_type).failures += 1;
    total_failures += 1;

    if total_failures > MAX_PRINTED_FAILURES return;

    shown := ifx original.count > 200 then tprint("%...", slice(original, 0, 200)) else original;

    print("FAIL % row % (%): %\n     original: %\n", check.label, row, check.sql_type, message, shown);
}

// Fetches the column in both formats, decodes both as `T`, and sends each distinct value back.
check_type :: (check: *Check, $T: Type, sql_type: string) {
    check.sql_type = sql_type;

    db := check.db;
    text_result, text_error := Pg.query_dynamic(db, check.select_sql);
    binary_result, binary_error := Pg.query_binary(db, check.select_sql);

    defer Pg.free_result(*text_result);
    defer Pg.free_result(*binary_result);

    if text_error.kind != .NONE || binary_error.kind != .NONE {
        fail(check, 0, "", tprint("fetch failed: % / %", Pg.error_text(text_error), Pg.error_text(binary_error)));
        return;
    }

    if text_result.row_count != binary_result.row_count {
        fail(check, 0, "", tprint("% rows as text, % as binary", text_result.row_count, binary_result.row_count));
        return;
    }

    stats := stats_for(sql_type);

    for row: 0..text_result.row_count - 1 {
        auto_release_temp();

        text_bytes, text_null := Pg.raw_value(*text_result, row, 0);
        _, binary_null := Pg.raw_value(*binary_result, row, 0);

        if text_null != binary_null {
            fail(check, row, "", "NULL in one format only");
            continue;
        }

        stats = stats_for(sql_type);

        if text_null {
            stats.nulls += 1;
            continue;
        }

        stats.values += 1;
        original := cast(string) text_bytes;

        // A column holds the same value many times in the regression tables: send each back once.
        key := tprint("%|%", sql_type, original);

        if table_contains(*seen, key) continue;

        table_add(*seen, copy_string(key), true);

        check_value(check, *text_result, *binary_result, row, original, T);
    }
}

check_value :: (check: *Check, text_result: *Pg.Result, binary_result: *Pg.Result, row: s64, original: string, $T: Type) {
    from_text, text_error := Pg.get(text_result, row, 0, T);
    from_binary, binary_error := Pg.get(binary_result, row, 0, T);

    if text_error.kind != .NONE || binary_error.kind != .NONE {
        // Arrays of more than one dimension have no Jai shape: counted, not checked.
        if contains(text_error.message, "multi-dimensional") && contains(binary_error.message, "multi-dimensional") {
            stats_for(check.sql_type).skipped += 1;
        } else {
            fail(check, row, original, tprint("decode failed: text: % / binary: %", Pg.error_text(text_error), Pg.error_text(binary_error)));
        }

        Pg.free_error(*text_error);
        Pg.free_error(*binary_error);

        return;
    }

    stats_for(check.sql_type).distinct += 1;

    if !same(from_text, from_binary) {
        fail(check, row, original, tprint("text decodes to % but binary to %", from_text, from_binary));
        return;
    }

    sql_type := check.sql_type;
    array := ends_with(sql_type, "[]");
    sql: string;

    if array sql = tprint("select to_jsonb($1::%) = to_jsonb($2::%), $1::%::text, $2::%::text", sql_type, sql_type, sql_type, sql_type);
    else sql = tprint("select $1::% is not distinct from $2::%, $1::%::text, $2::%::text", sql_type, sql_type, sql_type, sql_type);

    sent, error := Pg.query_dynamic(check.db, sql, from_binary, original);

    defer Pg.free_result(*sent);

    if error.kind != .NONE {
        fail(check, row, original, tprint("the server refused the value sent back (%): %", from_binary, Pg.error_text(error)));
        Pg.free_error(*error);
        return;
    }

    equal := get_or(*sent, 0, bool, false);
    sent_text := get_or(*sent, 1, string, "");
    original_text := get_or(*sent, 2, string, "");

    // Lower bounds are dropped by the client's array type, so their text cannot match.
    text_matters := check.exact_text && !(array && begins_with(original_text, "["));

    if !equal || (text_matters && sent_text != original_text) {
        fail(check, row, original, tprint("sent back as %, the server sees %", from_binary, sent_text));
    }
}

get_or :: (result: *Pg.Result, column: s64, $T: Type, default: T) -> T {
    value, error := Pg.get(result, 0, column, T);

    if error.kind != .NONE {
        Pg.free_error(*error);
        return default;
    }

    return value;
}

// Equality of two decoded values; floats by their bits, so -0.0 differs from 0.0 and NaN equals NaN.
same :: (a: $T, b: T) -> bool {
    #if T == string {
        return a == b;
    } else #if T == Pg.Numeric || T == Pg.Json_Raw {
        return a.text == b.text;
    } else #if T == [] u8 {
        return a.count == b.count && memcmp(a.data, b.data, a.count) == 0;
    } else #if type_info(T).type == .FLOAT {
        if a != a return b != b;
        return memcmp(*a, *b, size_of(T)) == 0;
    } else #if type_info(T).type == .ARRAY {
        if a.count != b.count return false;

        for 0..a.count - 1 if !same(a[it], b[it]) return false;

        return true;
    } else #if type_info(T).type == .STRUCT && (cast(*Type_Info_Struct) type_info(T)).polymorph_source_struct == type_info(Pg.Nullable) {
        if a.null != b.null return false;
        return a.null || same(a.value, b.value);
    } else {
        return memcmp(*a, *b, size_of(T)) == 0;
    }
}

// json and jsonb

Json_Outcome :: enum {
    CHECKED;
    REJECTED; // the server does not accept the text as this type
    FAILED;
}

check_json_column :: (check: *Check, kind: string) {
    check.sql_type = kind;

    result, error := Pg.query_dynamic(check.db, check.select_sql);

    defer Pg.free_result(*result);

    if error.kind != .NONE {
        fail(check, 0, "", tprint("fetch failed: %", Pg.error_text(error)));
        return;
    }

    for row: 0..result.row_count - 1 {
        auto_release_temp();

        bytes, is_null := Pg.raw_value(*result, row, 0);
        stats := stats_for(kind);

        if is_null {
            stats.nulls += 1;
            continue;
        }

        stats.values += 1;
        original := cast(string) bytes;
        key := tprint("%|%", kind, original);

        if table_contains(*seen, key) continue;

        table_add(*seen, copy_string(key), true);

        outcome := check_json_text(check.db, kind, original, tprint("% row %", check.label, row));
        if outcome == .REJECTED print("% row %: the server's own output was refused as %\n", check.label, row, kind);

    }
}

// Sends `text` as a json or jsonb literal, decodes the result in both formats, writes the decoded
// document back as a parameter and compares what the server holds.
check_json_text :: (db: *Pg.Connection, kind: string, text: string, label: string) -> Json_Outcome {
    check := Check.{ db = db, label = label, sql_type = kind };
    sql := tprint("select $1::%", kind);

    text_result, error := Pg.query_dynamic(db, sql, text);

    defer Pg.free_result(*text_result);

    if error.kind == .SERVER {
        Pg.free_error(*error);
        return .REJECTED;
    }

    if error.kind != .NONE {
        fail(*check, 0, text, Pg.error_text(error));
        return .FAILED;
    }

    binary_result, binary_error := Pg.query_binary(db, sql, text);

    defer Pg.free_result(*binary_result);

    if binary_error.kind != .NONE {
        fail(*check, 0, text, tprint("binary fetch failed: %", Pg.error_text(binary_error)));
        return .FAILED;
    }

    stats_for(kind).distinct += 1;

    raw_text := get_or(*text_result, 0, Pg.Json_Raw, .{});
    raw_binary := get_or(*binary_result, 0, Pg.Json_Raw, .{});

    if raw_text.text != raw_binary.text {
        fail(*check, 0, text, tprint("text gives % but binary %", raw_text.text, raw_binary.text));
        return .FAILED;
    }

    // json keeps the text it was given.
    if kind == "json" && raw_text.text != text {
        fail(*check, 0, text, tprint("json came back as %", raw_text.text));
        return .FAILED;
    }

    from_text, text_error := Pg.get(*text_result, 0, 0, Pg.Json(Json.Json_Value));
    from_binary, from_binary_error := Pg.get(*binary_result, 0, 0, Pg.Json(Json.Json_Value));

    if text_error.kind != .NONE || from_binary_error.kind != .NONE {
        // json keeps the text of a lone surrogate escape ("\ud83d"), which no Json_Value string
        // can hold: such a document is read as Json_Raw.
        if contains(text_error.message, "unpaired surrogate") && contains(from_binary_error.message, "unpaired surrogate") {
            unpaired_surrogates += 1;
            Pg.free_error(*text_error);
            Pg.free_error(*from_binary_error);
            return .CHECKED;
        }

        fail(*check, 0, text, tprint("decode failed: text: % / binary: %", Pg.error_text(text_error), Pg.error_text(from_binary_error)));

        return .FAILED;
    }

    if !json_same(*from_text.value, *from_binary.value) {
        fail(*check, 0, text, "text and binary decode to different documents");
        return .FAILED;
    }

    // The document, written back by the client.
    echo, echo_error := Pg.query_dynamic(db, sql, Pg.json(from_text.value));

    defer Pg.free_result(*echo);

    if echo_error.kind != .NONE {
        fail(*check, 0, text, tprint("the server refused the document sent back: %", Pg.error_text(echo_error)));
        return .FAILED;
    }

    again, again_error := Pg.get(*echo, 0, 0, Pg.Json(Json.Json_Value));

    if again_error.kind != .NONE || !json_same(*from_text.value, *again.value) {
        fail(*check, 0, text, tprint("the document sent back reads as %", get_or(*echo, 0, string, "")));
        return .FAILED;
    }

    // And the server agrees it is the same document. A json text the server cannot turn into
    // jsonb (a \u0000 escape) has no such comparison.
    compare := "select $1::jsonb = $2::jsonb, $1::jsonb::text = $2::jsonb::text";
    sent, compare_error := Pg.query_dynamic(db, compare, Pg.json(from_text.value), text);

    defer Pg.free_result(*sent);

    if compare_error.kind == .SERVER && kind == "json" {
        json_without_jsonb += 1;
        Pg.free_error(*compare_error);
        return .CHECKED;
    }

    if compare_error.kind != .NONE {
        fail(*check, 0, text, tprint("the server refused the comparison: %", Pg.error_text(compare_error)));
        return .FAILED;
    }

    if !get_or(*sent, 0, bool, false) || !get_or(*sent, 1, bool, false) {
        fail(*check, 0, text, tprint("the server sees a different document than %", raw_text.text));
        return .FAILED;
    }

    return .CHECKED;
}

json_same :: (a: *Json.Json_Value, b: *Json.Json_Value) -> bool {
    if a.kind != b.kind return false;

    if a.kind == {
        case .BOOL; return a.boolean == b.boolean;
        case .NUMBER; return a.text == b.text;
        case .STRING; return a.text == b.text;

        case .ARRAY;
            if a.items.count != b.items.count return false;
            for 0..a.items.count - 1 if !json_same(*a.items[it], *b.items[it]) return false;

        case .OBJECT;
            if a.members.count != b.members.count return false;

            for 0..a.members.count - 1 {
                if a.members[it].key != b.members[it].key return false;
                if !json_same(*a.members[it].value, *b.members[it].value) return false;
            }
    }

    return true;
}

// One literal per line: the type (json, jsonb, or any), a tab, then the text with \\, \n, \r and
// \t escaped.
check_literals :: (db: *Pg.Connection, path: string) {
    content, ok := read_entire_file(path, log_errors = false);

    if !ok {
        print("cannot read %\n", path);
        total_failures += 1;
        return;
    }

    for split(content, "\n") {
        auto_release_temp();

        tab := find_index_from_left(it, #char "\t");
        if tab < 0 continue;

        kind := slice(it, 0, tab);
        text := unescape_literal(advance(it, tab + 1));
        label := tprint("literal %", it_index + 1);
        checked := false;

        for candidate: string.["json", "jsonb"] {
            if kind != "any" && kind != candidate continue;
            if check_json_text(db, candidate, text, label) == .CHECKED checked = true;
        }

        if !checked rejected_literals += 1;
    }
}

unescape_literal :: (line: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    i := 0;

    while i < line.count {
        c := line[i];

        if c == #char "\\" && i + 1 < line.count {
            i += 1;
            n := line[i];

            if n == #char "n" append(*builder, "\n");
            else if n == #char "r" append(*builder, "\r");
            else if n == #char "t" append(*builder, "\t");
            else append(*builder, slice(line, i, 1));
        } else {
            append(*builder, slice(line, i, 1));
        }

        i += 1;
    }

    return builder_to_string(*builder,, temp);
}

report :: () {
    print("\n%\n", row_text("type", "values", "distinct", "nulls", "skipped", "failures"));

    totals: Type_Stats;

    for all_stats {
        print("%\n", row_text(it.name, tprint("%", it.values), tprint("%", it.distinct), tprint("%", it.nulls), tprint("%", it.skipped), tprint("%", it.failures)));
        totals.values += it.values;
        totals.distinct += it.distinct;
        totals.nulls += it.nulls;
        totals.skipped += it.skipped;
        totals.failures += it.failures;
    }

    print("%\n", row_text("total", tprint("%", totals.values), tprint("%", totals.distinct), tprint("%", totals.nulls), tprint("%", totals.skipped), tprint("%", totals.failures)));

    if rejected_literals print("json literals the server rejects: %\n", rejected_literals);
    if json_without_jsonb print("json documents with no jsonb form (compared as json only): %\n", json_without_jsonb);

    if unpaired_surrogates print("json documents with a lone surrogate escape (read as Json_Raw only): %\n", unpaired_surrogates);

    if unsupported_columns.count {
        print("columns of types the client does not decode: %\n", unsupported_columns.count);
        for unsupported_columns if it_index < 15 print("  %\n", it);
    }

    if total_failures print("FAILED: % failures\n", total_failures);
    else print("ok\n");
}

row_text :: (name: string, values: string, distinct: string, nulls: string, skipped: string, failures: string) -> string {
    return tprint("%1%2%3%4%5%6", pad(name, 14), pad(values, 10), pad(distinct, 10), pad(nulls, 8), pad(skipped, 9), failures);
}

pad :: (text: string, width: s64) -> string {
    if text.count >= width return text;

    padding := talloc_string(width - text.count);
    memset(padding.data, #char " ", padding.count);

    return tprint("%1%2", text, padding);
}
