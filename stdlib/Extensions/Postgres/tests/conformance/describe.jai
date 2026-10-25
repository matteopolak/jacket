// psql's \d family and the other catalog-describing commands (\sv, \sf, \z, \l, \dconfig), for the
// psql shim (psql.jai).
// The catalog queries are psql's (PostgreSQL License: see COPYRIGHT.postgresql).

// The command `name` (without the backslash) with its arguments still in the scanner. `handled`
// is false for a name this file does not know.
describe_command :: (l: *Loop, full_name: string) -> Command_Status, handled: bool {
    auto_release_temp();

    // PostgreSQL 18: an `x` after the command letters (before or after `+` and `S`) shows this
    // command's result in expanded mode. `\dx` itself lists extensions.
    name, expanded := without_expanded_flag(full_name);

    if expanded {
        saved := psql.popt.expanded;
        psql.popt.expanded = .ON;
        defer psql.popt.expanded = saved;

        return describe_command_inner(l, name);
    }

    return describe_command_inner(l, name);
}

without_expanded_flag :: (name: string) -> string, bool {
    if !at_least(180000) || !(begins_with(name, "d") || begins_with(name, "z")) return name, false;

    end := name.count;
    while end > 0 && (name[end - 1] == #char "+" || name[end - 1] == #char "S") end -= 1;

    if end < 2 || name[end - 1] != #char "x" return name, false;
    if slice(name, 0, end) == "dx" return name, false;

    return join(slice(name, 0, end - 1), slice(name, end, name.count - end),, temp), true;
}

describe_command_inner :: (l: *Loop, name: string) -> Command_Status, handled: bool {

    s := *l.scanner;
    verbose := contains(name, "+");
    show_system := contains(name, "S");

    if name == "sf" || name == "sf+" || name == "sv" || name == "sv+" {
        return show_source(l, name), true;
    }

    arg, found := next_arg(s, .NORMAL, true);
    pattern := Pattern.{ arg, found };

    ok := true;
    status := Command_Status.SKIP_LINE;

    if name == "z" || name == "zS" {
        ok = permissions_list(pattern, show_system);
    } else if name == "l" || name == "l+" || name == "list" || name == "list+" {
        ok = list_all_dbs(pattern, verbose);
    } else if name == "lo_list" || name == "lo_list+" {
        ok = list_large_objects(verbose);
    } else if begins_with(name, "d") {
        handled: bool;
        ok, handled = describe_d(s, name, pattern, verbose, show_system);
        if !handled return .ERROR, false;
    } else {
        return .ERROR, false;
    }

    return ifx ok then status else .ERROR, true;
}

#scope_file

// Building queries and reading their results

put :: (b: *String_Builder, text: string) {
    append(b, text);
}

putf :: (b: *String_Builder, format: string, args: ..Any) {
    print_to_builder(b, format, ..args);
}

new_sql :: () -> String_Builder {
    b: String_Builder;
    b.allocator = temp;
    return b;
}

sql_text :: (b: *String_Builder) -> string {
    return builder_to_string(b,, temp);
}

cell :: (r: *Pg.Result, row: s64, col: s64) -> string {
    bytes, is_null := Pg.raw_value(r, row, col);
    if is_null return "";
    return cast(string) bytes;
}

is_null :: (r: *Pg.Result, row: s64, col: s64) -> bool {
    _, missing := Pg.raw_value(r, row, col);
    return missing;
}

is_true :: (r: *Pg.Result, row: s64, col: s64) -> bool {
    return cell(r, row, col) == "t";
}

to_int :: (text: string) -> s64 {
    value: s64;
    negative := false;

    for text {
        c := it;
        if it_index == 0 && c == #char "-" {
            negative = true;
            continue;
        }

        if c < #char "0" || c > #char "9" break;

        value = value * 10 + (c - #char "0");
    }

    return ifx negative then -value else value;
}

first_char :: (text: string) -> u8 {
    return ifx text.count > 0 then text[0] else 0;
}

at_least :: (version: s32) -> bool {
    return psql.db.server_version >= version;
}

// A table assembled by hand, as printTableContent is

Content :: struct {
    title: string;
    headers: [..] string;
    aligns: [..] u8;
    cells: [..] string;
    footers: [..] string;
    default_footer: bool;
}

content_init :: (c: *Content, title: string) {
    c.* = .{};
    c.title = title;
    c.headers.allocator = temp;
    c.aligns.allocator = temp;
    c.cells.allocator = temp;
    c.footers.allocator = temp;
}

add_header :: (c: *Content, name: string, align: u8 = #char "l") {
    array_add(*c.headers, name);
    array_add(*c.aligns, align);
}

add_cell :: (c: *Content, text: string) {
    array_add(*c.cells, text);
}

add_footer :: (c: *Content, text: string) {
    array_add(*c.footers, text);
}

set_last_footer :: (c: *Content, text: string) {
    if c.footers.count == 0 return;
    c.footers[c.footers.count - 1] = text;
}

emit_content :: (c: *Content, options: Print_Options) {
    t: Table;
    t.title = c.title;
    t.headers = c.headers;
    t.align = c.aligns;
    t.cells = c.cells;
    t.footers = c.footers;
    t.rows = ifx c.headers.count > 0 then c.cells.count / c.headers.count else 0;
    t.default_footer = c.default_footer;
    emit_table(t, options);
}

emit_table :: (t: Table, options: Print_Options) {
    builder: String_Builder;

    print_table(*builder, t, options);
    text := builder_to_string(*builder);

    write_out(text);
    free(text);
}

// printQuery on a query result: `options` is the caller's copy of the print options.
print_result :: (r: *Pg.Result, title: string, options: Print_Options, footers: [] string = .[], default_footer := true) {
    auto_release_temp();

    t: Table;
    t.title = title;
    t.footers = footers;
    t.default_footer = default_footer;

    columns := r.columns.count;

    t.headers = NewArray(columns, string,, temp);
    t.align = NewArray(columns, u8,, temp);
    t.cells = NewArray(r.row_count * columns, string,, temp);
    t.nulls = NewArray(r.row_count * columns, bool,, temp);
    t.rows = r.row_count;

    for r.columns {
        t.headers[it_index] = it.name;
        t.align[it_index] = column_alignment(it.type_oid);
    }

    for row: 0..r.row_count - 1 {
        for col: 0..columns - 1 {
            bytes, missing := Pg.raw_value(r, row, col);
            t.cells[row * columns + col] = ifx missing then cast(string) bytes else display_bool(options, r.columns[col].type_oid, cast(string) bytes);
            t.nulls[row * columns + col] = missing;
        }
    }

    emit_table(t, options);
}

// Name patterns (psql's processSQLNamePattern)

Pattern :: struct {
    text: string;
    set: bool;
}

is_regex_special :: (c: u8) -> bool {
    return c == #char "|" || c == #char "*" || c == #char "+" || c == #char "?" || c == #char "("
        || c == #char ")" || c == #char "[" || c == #char "]" || c == #char "{" || c == #char "}"
        || c == #char "." || c == #char "^" || c == #char "$" || c == #char "\\";
}

// Shell-style pattern to up to three anchored regular expressions. `parts` says how many dotted
// names are split off (1: name only, 2: schema.name, 3: db.schema.name); extra dots stay in the
// name. The database part is returned as the literal text before the first dot.
pattern_to_regex :: (pattern: string, force_escape: bool, parts: s64) -> db: string, schema: string, name: string, dotcnt: s64 {
    bufs: [3] String_Builder;
    for *bufs it.allocator = temp;

    left_literal := new_sql();
    cur := 0;
    max_buf := parts - 1;
    dotcnt := 0;
    in_quotes := false;
    left := true;

    put(*bufs[0], "^(");

    i := 0;
    while i < pattern.count {
        ch := pattern[i];

        if ch == #char "\"" {
            if in_quotes && i + 1 < pattern.count && pattern[i + 1] == #char "\"" {
                put(*bufs[cur], "\"");
                if left put(*left_literal, "\"");
                i += 1;
            } else {
                in_quotes = !in_quotes;
            }

            i += 1;
        } else if !in_quotes && ch >= #char "A" && ch <= #char "Z" {
            lowered: string = .{ 1, *ch };
            lc: u8 = ch + 32;

            lowered.data = *lc;
            put(*bufs[cur], lowered);
            if left put(*left_literal, lowered);

            i += 1;
        } else if !in_quotes && ch == #char "*" {
            put(*bufs[cur], ".*");
            if left put(*left_literal, "*");
            i += 1;
        } else if !in_quotes && ch == #char "?" {
            put(*bufs[cur], ".");
            if left put(*left_literal, "?");
            i += 1;
        } else if !in_quotes && ch == #char "." {
            left = false;
            dotcnt += 1;

            if cur < max_buf {
                put(*bufs[cur], ")$");
                cur += 1;
                put(*bufs[cur], "^(");
            } else {
                put(*bufs[cur], ".");
            }

            i += 1;
        } else if ch == #char "$" {
            put(*bufs[cur], "\\$");
            if left put(*left_literal, "$");
            i += 1;
        } else {
            one: string = .{ 1, *ch };

            if (in_quotes || force_escape) && is_regex_special(ch) {
                put(*bufs[cur], "\\");
            } else if ch == #char "[" && i + 1 < pattern.count && pattern[i + 1] == #char "]" {
                put(*bufs[cur], "\\");
            }

            put(*bufs[cur], one);
            if left put(*left_literal, one);

            i += 1;
        }
    }

    put(*bufs[cur], ")$");

    db, schema, name: string;

    name = sql_text(*bufs[cur]);
    cur -= 1;

    if parts >= 2 && cur >= 0 {
        schema = sql_text(*bufs[cur]);
        cur -= 1;
    }

    if parts >= 3 && cur >= 0 {
        db = sql_text(*left_literal);
    }

    return db, schema, name, dotcnt;
}

// Appends the WHERE clauses for a pattern; returns whether any was added.
process_pattern :: (b: *String_Builder, pattern: Pattern, have_where: bool, force_escape: bool, schemavar: string, namevar: string, altnamevar: string, visibility: string, parts: s64) -> added: bool, db: string, dotcnt: s64 {
    added := false;
    where := have_where;

    if !pattern.set {
        if visibility.count {
            put(b, ifx where then "  AND " else "WHERE ");
            put(b, visibility);
            put(b, "\n");
            added = true;
        }

        return added, "", 0;
    }

    db, schema, name, dotcnt := pattern_to_regex(pattern.text, force_escape, ifx schemavar.count then parts else 1);

    if namevar.count && name.count > 2 {
        if name != "^(.*)$" {
            put(b, ifx where then "  AND " else "WHERE ");
            where = true;
            added = true;

            literal := escape_literal(name);

            if altnamevar.count {
                putf(b, "(% OPERATOR(pg_catalog.~) %", namevar, literal);
                put(b, " COLLATE pg_catalog.default");
                putf(b, "\n        OR % OPERATOR(pg_catalog.~) %", altnamevar, literal);
                put(b, " COLLATE pg_catalog.default)\n");
            } else {
                putf(b, "% OPERATOR(pg_catalog.~) %", namevar, literal);
                put(b, " COLLATE pg_catalog.default\n");
            }
        }
    }

    if schemavar.count && schema.count > 2 {
        if schema != "^(.*)$" {
            put(b, ifx where then "  AND " else "WHERE ");
            where = true;
            added = true;

            putf(b, "% OPERATOR(pg_catalog.~) %", schemavar, escape_literal(schema));
            put(b, " COLLATE pg_catalog.default\n");
        }
    } else if visibility.count {
        put(b, ifx where then "  AND " else "WHERE ");
        where = true;
        added = true;
        put(b, visibility);
        put(b, "\n");
    }

    return added, db, dotcnt;
}

// processSQLNamePattern plus the checks on how many dotted names the pattern has.
validate_pattern :: (b: *String_Builder, pattern: Pattern, have_where: bool, force_escape: bool, schemavar: string, namevar: string, altnamevar: string, visibility: string, maxparts: s64) -> ok: bool, added: bool {
    added, db, dotcnt := process_pattern(b, pattern, have_where, force_escape, schemavar, namevar, altnamevar, visibility, maxparts);

    if dotcnt >= maxparts {
        print_error(tprint("improper qualified name (too many dotted names): %", pattern.text));
        return false, added;
    }

    if maxparts > 1 && dotcnt == maxparts - 1 {
        if psql.config.database != db {
            print_error(tprint("cross-database references are not implemented: %", pattern.text));
            return false, added;
        }
    }

    return true, added;
}

// \d NAME

// PostgreSQL 18 names the kind when a listing asks for exactly one.
relations_title :: (tables: bool, views: bool, matviews: bool, indexes: bool, sequences: bool, foreign: bool) -> string {
    count := 0;
    for bool.[tables, views, matviews, indexes, sequences, foreign] if it count += 1;

    if count != 1 || !at_least(180000) return "List of relations";
    if tables return "List of tables";
    if views return "List of views";
    if matviews return "List of materialized views";
    if indexes return "List of indexes";
    if sequences return "List of sequences";

    return "List of foreign tables";
}

describe_table_details :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT c.oid,\n  n.nspname,\n  c.relname\nFROM pg_catalog.pg_class c\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\n");

    if !show_system && !pattern.set {
        put(*b, "WHERE n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");
    }

    if !validate_pattern(*b, pattern, !show_system && !pattern.set, false, "n.nspname", "c.relname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 2, 3;");

    r := psql_exec(sql_text(*b));
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 {
        if !psql.quiet {
            if pattern.set print_error(tprint("Did not find any relation named \"%\".", pattern.text));
            else print_error("Did not find any relations.");
        }

        return false;
    }

    for 0..r.row_count - 1 {
        if !describe_one_table(cell(r, it, 1), cell(r, it, 2), cell(r, it, 0), verbose) return false;
    }

    return true;
}

Relkind :: struct {
    RELATION :: #char "r";
    INDEX :: #char "i";
    SEQUENCE :: #char "S";
    TOASTVALUE :: #char "t";
    VIEW :: #char "v";
    MATVIEW :: #char "m";
    COMPOSITE_TYPE :: #char "c";
    FOREIGN_TABLE :: #char "f";
    PARTITIONED_TABLE :: #char "p";
    PARTITIONED_INDEX :: #char "I";
}

// What pg_class says about the relation being described.
Table_Info :: struct {
    schemaname: string;
    relationname: string;
    oid: string;
    verbose: bool;
    checks: s64;
    relkind: u8;
    hasindex: bool;
    hasrules: bool;
    hastriggers: bool;
    rowsecurity: bool;
    forcerowsecurity: bool;
    hasoids: bool;
    ispartition: bool;
    reloptions: string;
    tablespace: string;
    reloftype: string;
    relpersistence: u8;
    relreplident: u8;
    relam: string;
    has_relam: bool;
}

// Where each optional column landed in the attribute query, or -1 when it was left out.
Column_Layout :: struct {
    show_details: bool;
    attname: s64 = -1;
    atttype: s64 = -1;
    attrdef: s64 = -1;
    attnotnull: s64 = -1;
    attcoll: s64 = -1;
    attidentity: s64 = -1;
    attgenerated: s64 = -1;
    isindexkey: s64 = -1;
    indexdef: s64 = -1;
    fdwopts: s64 = -1;
    attstorage: s64 = -1;
    attcompression: s64 = -1;
    attstattarget: s64 = -1;
    attdescr: s64 = -1;
}

read_table_info :: (info: *Pg.Result, schemaname: string, relationname: string, oid: string, verbose: bool) -> Table_Info {
    t: Table_Info;
    t.schemaname = schemaname;
    t.relationname = relationname;
    t.oid = oid;
    t.verbose = verbose;
    t.checks = to_int(cell(info, 0, 0));
    t.relkind = first_char(cell(info, 0, 1));
    t.hasindex = is_true(info, 0, 2);
    t.hasrules = is_true(info, 0, 3);
    t.hastriggers = is_true(info, 0, 4);
    t.rowsecurity = is_true(info, 0, 5);
    t.forcerowsecurity = is_true(info, 0, 6);
    t.hasoids = is_true(info, 0, 7);
    t.ispartition = is_true(info, 0, 8);
    t.reloptions = cell(info, 0, 9);
    t.tablespace = cell(info, 0, 10);
    t.reloftype = cell(info, 0, 11);
    t.relpersistence = first_char(cell(info, 0, 12));
    t.relreplident = first_char(cell(info, 0, 13));
    t.relam = cell(info, 0, 14);
    t.has_relam = !is_null(info, 0, 14);

    return t;
}

has_column_details :: (relkind: u8) -> bool {
    return relkind == Relkind.RELATION || relkind == Relkind.VIEW || relkind == Relkind.MATVIEW
        || relkind == Relkind.FOREIGN_TABLE || relkind == Relkind.COMPOSITE_TYPE || relkind == Relkind.PARTITIONED_TABLE;
}

// Relations that keep rows (or their index/toast counterparts) and so list constraints and parents.
has_table_footers :: (relkind: u8) -> bool {
    return relkind == Relkind.RELATION || relkind == Relkind.MATVIEW || relkind == Relkind.FOREIGN_TABLE
        || relkind == Relkind.PARTITIONED_TABLE || relkind == Relkind.PARTITIONED_INDEX || relkind == Relkind.TOASTVALUE;
}

is_index_kind :: (relkind: u8) -> bool {
    return relkind == Relkind.INDEX || relkind == Relkind.PARTITIONED_INDEX;
}

has_compression_column :: (relkind: u8) -> bool {
    return relkind == Relkind.RELATION || relkind == Relkind.PARTITIONED_TABLE || relkind == Relkind.MATVIEW;
}

has_stats_target_column :: (relkind: u8) -> bool {
    return relkind == Relkind.RELATION || relkind == Relkind.INDEX || relkind == Relkind.PARTITIONED_INDEX
        || relkind == Relkind.MATVIEW || relkind == Relkind.FOREIGN_TABLE || relkind == Relkind.PARTITIONED_TABLE;
}

// Appends one column to the attribute query and returns where it landed.
add_query_column :: (b: *String_Builder, cols: *s64, sql: string) -> s64 {
    put(b, sql);
    cols.* += 1;

    return cols.* - 1;
}

add_detail_columns :: (b: *String_Builder, cols: *s64, l: *Column_Layout) {
    put(b, ",\n  (SELECT pg_catalog.pg_get_expr(d.adbin, d.adrelid, true)\n   FROM pg_catalog.pg_attrdef d\n   WHERE d.adrelid = a.attrelid AND d.adnum = a.attnum AND a.atthasdef),\n  a.attnotnull");
    l.attrdef = cols.*;
    l.attnotnull = cols.* + 1;
    cols.* += 2;

    l.attcoll = add_query_column(b, cols, ",\n  (SELECT c.collname FROM pg_catalog.pg_collation c, pg_catalog.pg_type t\n   WHERE c.oid = a.attcollation AND t.oid = a.atttypid AND a.attcollation <> t.typcollation) AS attcollation");
    l.attidentity = add_query_column(b, cols, ",\n  a.attidentity");
    l.attgenerated = add_query_column(b, cols, ",\n  a.attgenerated");
}

add_verbose_columns :: (b: *String_Builder, cols: *s64, l: *Column_Layout, relkind: u8) {
    l.attstorage = add_query_column(b, cols, ",\n  a.attstorage");

    if !psql.hide_toast_compression && has_compression_column(relkind) {
        l.attcompression = add_query_column(b, cols, ",\n  a.attcompression AS attcompression");
    }

    if has_stats_target_column(relkind) {
        l.attstattarget = add_query_column(b, cols, ",\n  CASE WHEN a.attstattarget=-1 THEN NULL ELSE a.attstattarget END AS attstattarget");
    }

    if l.show_details {
        l.attdescr = add_query_column(b, cols, ",\n  pg_catalog.col_description(a.attrelid, a.attnum)");
    }
}

build_column_query :: (b: *String_Builder, t: *Table_Info) -> Column_Layout {
    l: Column_Layout;
    l.show_details = has_column_details(t.relkind);
    cols := 0;

    l.attname = add_query_column(b, *cols, "SELECT a.attname");
    l.atttype = add_query_column(b, *cols, ",\n  pg_catalog.format_type(a.atttypid, a.atttypmod)");

    if l.show_details add_detail_columns(b, *cols, *l);

    if is_index_kind(t.relkind) {
        l.isindexkey = add_query_column(b, *cols, tprint(",\n  CASE WHEN a.attnum <= (SELECT i.indnkeyatts FROM pg_catalog.pg_index i WHERE i.indexrelid = '%') THEN 'yes' ELSE 'no' END AS is_key", t.oid));
        l.indexdef = add_query_column(b, *cols, ",\n  pg_catalog.pg_get_indexdef(a.attrelid, a.attnum, TRUE) AS indexdef");
    }

    if t.relkind == Relkind.FOREIGN_TABLE {
        l.fdwopts = add_query_column(b, *cols, ",\n  CASE WHEN attfdwoptions IS NULL THEN '' ELSE   '(' || pg_catalog.array_to_string(ARRAY(SELECT pg_catalog.quote_ident(option_name) || ' ' || pg_catalog.quote_literal(option_value)  FROM   pg_catalog.pg_options_to_table(attfdwoptions)), ', ') || ')' END AS attfdwoptions");
    }

    if t.verbose add_verbose_columns(b, *cols, *l, t.relkind);

    putf(b, "\nFROM pg_catalog.pg_attribute a\nWHERE a.attrelid = '%' AND a.attnum > 0 AND NOT a.attisdropped\nORDER BY a.attnum;", t.oid);

    return l;
}

table_title :: (t: *Table_Info) -> string {
    qualified := tprint("\"%.%\"", t.schemaname, t.relationname);
    unlogged := t.relpersistence == #char "u";

    if t.relkind == {
        case Relkind.RELATION;
            return tprint("% %", ifx unlogged then "Unlogged table" else "Table", qualified);
        case Relkind.VIEW;
            return tprint("View %", qualified);
        case Relkind.MATVIEW;
            return tprint("% %", ifx unlogged then "Unlogged materialized view" else "Materialized view", qualified);
        case Relkind.INDEX;
            return tprint("% %", ifx unlogged then "Unlogged index" else "Index", qualified);
        case Relkind.PARTITIONED_INDEX;
            return tprint("% %", ifx unlogged then "Unlogged partitioned index" else "Partitioned index", qualified);
        case Relkind.TOASTVALUE;
            return tprint("TOAST table %", qualified);
        case Relkind.COMPOSITE_TYPE;
            return tprint("Composite type %", qualified);
        case Relkind.FOREIGN_TABLE;
            return tprint("Foreign table %", qualified);
        case Relkind.PARTITIONED_TABLE;
            return tprint("% %", ifx unlogged then "Unlogged partitioned table" else "Partitioned table", qualified);
    }

    kind: string = .{ 1, *t.relkind };

    return tprint("?%? %", kind, qualified);
}

add_column_headers :: (cont: *Content, l: *Column_Layout) {
    add_header(cont, "Column");
    add_header(cont, "Type");

    if l.show_details {
        add_header(cont, "Collation");
        add_header(cont, "Nullable");
        add_header(cont, "Default");
    }

    if l.isindexkey >= 0 add_header(cont, "Key?");
    if l.indexdef >= 0 add_header(cont, "Definition");
    if l.fdwopts >= 0 add_header(cont, "FDW options");
    if l.attstorage >= 0 add_header(cont, "Storage");
    if l.attcompression >= 0 add_header(cont, "Compression");
    if l.attstattarget >= 0 add_header(cont, "Stats target");
    if l.attdescr >= 0 add_header(cont, "Description");
}

default_column_text :: (res: *Pg.Result, row: s64, l: *Column_Layout) -> string {
    identity := first_char(cell(res, row, l.attidentity));
    generated := first_char(cell(res, row, l.attgenerated));

    if identity == #char "a" return "generated always as identity";
    if identity == #char "d" return "generated by default as identity";
    if generated == #char "s" return tprint("generated always as (%) stored", cell(res, row, l.attrdef));
    if generated == #char "v" return tprint("generated always as (%)", cell(res, row, l.attrdef));

    return cell(res, row, l.attrdef);
}

storage_text :: (storage: u8) -> string {
    if storage == #char "p" return "plain";
    if storage == #char "m" return "main";
    if storage == #char "x" return "extended";
    if storage == #char "e" return "external";

    return "???";
}

compression_text :: (compression: u8) -> string {
    if compression == #char "p" return "pglz";
    if compression == #char "l" return "lz4";
    if compression == 0 return "";

    return "???";
}

add_column_row :: (cont: *Content, res: *Pg.Result, row: s64, l: *Column_Layout) {
    add_cell(cont, cell(res, row, l.attname));
    add_cell(cont, cell(res, row, l.atttype));

    if l.show_details {
        add_cell(cont, cell(res, row, l.attcoll));
        add_cell(cont, ifx is_true(res, row, l.attnotnull) then "not null" else "");
        add_cell(cont, default_column_text(res, row, l));
    }

    if l.isindexkey >= 0 add_cell(cont, cell(res, row, l.isindexkey));
    if l.indexdef >= 0 add_cell(cont, cell(res, row, l.indexdef));
    if l.fdwopts >= 0 add_cell(cont, cell(res, row, l.fdwopts));
    if l.attstorage >= 0 add_cell(cont, storage_text(first_char(cell(res, row, l.attstorage))));
    if l.attcompression >= 0 add_cell(cont, compression_text(first_char(cell(res, row, l.attcompression))));
    if l.attstattarget >= 0 add_cell(cont, cell(res, row, l.attstattarget));
    if l.attdescr >= 0 add_cell(cont, cell(res, row, l.attdescr));
}

add_partition_footers :: (cont: *Content, t: *Table_Info) -> bool {
    b := new_sql();

    put(*b, "SELECT inhparent::pg_catalog.regclass,\n  pg_catalog.pg_get_expr(c.relpartbound, c.oid),\n  inhdetachpending");
    if t.verbose put(*b, ",\n  pg_catalog.pg_get_partition_constraintdef(c.oid)");

    putf(*b, "\nFROM pg_catalog.pg_class c JOIN pg_catalog.pg_inherits i ON c.oid = inhrelid\nWHERE c.oid = '%';", t.oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    pending := ifx cell(result, 0, 2) == "t" then " DETACH PENDING" else "";

    add_footer(cont, tprint("Partition of: %1 %2%3", cell(result, 0, 0), cell(result, 0, 1), pending));

    if t.verbose {
        constraint := ifx is_null(result, 0, 3) then "" else cell(result, 0, 3);
        if constraint.count == 0 add_footer(cont, "No partition constraint");
        else add_footer(cont, tprint("Partition constraint: %", constraint));
    }

    return true;
}

add_partition_key_footer :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT pg_catalog.pg_get_partkeydef('%'::pg_catalog.oid);", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 1 add_footer(cont, tprint("Partition key: %", cell(result, 0, 0)));

    return true;
}

add_owning_table_footer :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT n.nspname, c.relname\nFROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\nWHERE reltoastrelid = '%';", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 1 add_footer(cont, tprint("Owning table: \"%.%\"", cell(result, 0, 0), cell(result, 0, 1)));

    return true;
}

// The one-line summary under an index: its kind, table and state.
add_index_summary_footer :: (cont: *Content, t: *Table_Info) -> bool {
    b := new_sql();

    putf(*b, "SELECT i.indisunique, i.indisprimary, i.indisclustered, i.indisvalid,\n  (NOT i.indimmediate) AND EXISTS (SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid = i.indrelid AND conindid = i.indexrelid AND contype IN ('p','u','x') AND condeferrable) AS condeferrable,\n  (NOT i.indimmediate) AND EXISTS (SELECT 1 FROM pg_catalog.pg_constraint WHERE conrelid = i.indrelid AND conindid = i.indexrelid AND contype IN ('p','u','x') AND condeferred) AS condeferred,\ni.indisreplident,\ni.indnullsnotdistinct,\n  a.amname, c2.relname, pg_catalog.pg_get_expr(i.indpred, i.indrelid, true)\nFROM pg_catalog.pg_index i, pg_catalog.pg_class c, pg_catalog.pg_class c2, pg_catalog.pg_am a\nWHERE i.indexrelid = c.oid AND c.oid = '%' AND c.relam = a.oid\nAND i.indrelid = c2.oid;", t.oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);
    if result.row_count != 1 return false;

    prefix: string;

    if is_true(result, 0, 1) {
        prefix = "primary key, ";
    } else if is_true(result, 0, 0) {
        prefix = "unique";
        if is_true(result, 0, 7) prefix = tprint("% nulls not distinct", prefix);
        prefix = tprint("%, ", prefix);
    }

    text := tprint("%, for table \"%.%\"", cell(result, 0, 8), t.schemaname, cell(result, 0, 9));
    text = tprint("%1%2", prefix, text);

    predicate := cell(result, 0, 10);
    if predicate.count text = tprint("%, predicate (%)", text, predicate);
    if is_true(result, 0, 2) text = tprint("%, clustered", text);
    if !is_true(result, 0, 3) text = tprint("%, invalid", text);
    if is_true(result, 0, 4) text = tprint("%, deferrable", text);
    if is_true(result, 0, 5) text = tprint("%, initially deferred", text);
    if is_true(result, 0, 6) text = tprint("%, replica identity", text);

    add_footer(cont, text);

    if t.relkind == Relkind.INDEX add_tablespace_footer(cont, t.relkind, t.tablespace, true);

    return true;
}

add_view_footers :: (cont: *Content, t: *Table_Info) -> bool {
    b := new_sql();

    putf(*b, "SELECT pg_catalog.pg_get_viewdef('%'::pg_catalog.oid, true);", t.oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "View definition:");
    add_footer(cont, copy_string(cell(result, 0, 0),, temp));

    if t.hasrules return add_view_rule_footers(cont, t.oid);

    return true;
}

add_view_rule_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT r.rulename, trim(trailing ';' from pg_catalog.pg_get_ruledef(r.oid, true))\nFROM pg_catalog.pg_rewrite r\nWHERE r.ev_class = '%' AND r.rulename != '_RETURN' ORDER BY 1;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Rules:");

    for 0..result.row_count - 1 {
        add_footer(cont, tprint(" %", advance(cell(result, it, 1), 12)));
    }

    return true;
}

// The footers that depend on the kind of relation.
add_kind_footers :: (cont: *Content, t: *Table_Info) -> bool {
    if t.ispartition && !add_partition_footers(cont, t) return false;
    if t.relkind == Relkind.PARTITIONED_TABLE && !add_partition_key_footer(cont, t.oid) return false;
    if t.relkind == Relkind.TOASTVALUE && !add_owning_table_footer(cont, t.oid) return false;

    if is_index_kind(t.relkind) {
        if !add_index_summary_footer(cont, t) return false;
    } else if has_table_footers(t.relkind) {
        if !add_table_footers(cont, t.oid, t.relkind, t.hasindex, t.checks, t.hastriggers, t.ispartition, t.rowsecurity, t.forcerowsecurity, t.hasrules) return false;
        if t.verbose && at_least(180000) && !add_not_null_footers(cont, t.oid, t.relkind) return false;
    }

    is_view := t.relkind == Relkind.VIEW || t.relkind == Relkind.MATVIEW;
    if is_view && t.verbose && !add_view_footers(cont, t) return false;

    return true;
}

describe_one_table :: (schemaname: string, relationname: string, oid: string, verbose: bool) -> bool {
    options := psql.popt;
    options.expanded = .OFF;

    b := new_sql();

    putf(*b, "SELECT c.relchecks, c.relkind, c.relhasindex, c.relhasrules, c.relhastriggers, c.relrowsecurity, c.relforcerowsecurity, false AS relhasoids, c.relispartition, %, c.reltablespace, CASE WHEN c.reloftype = 0 THEN '' ELSE c.reloftype::pg_catalog.regtype::pg_catalog.text END, c.relpersistence, c.relreplident, am.amname\nFROM pg_catalog.pg_class c\n LEFT JOIN pg_catalog.pg_class tc ON (c.reltoastrelid = tc.oid)\nLEFT JOIN pg_catalog.pg_am am ON (c.relam = am.oid)\nWHERE c.oid = '%';",
        ifx verbose then "pg_catalog.array_to_string(c.reloptions || array(select 'toast.' || x from pg_catalog.unnest(tc.reloptions) x), ', ')\n" else "''", oid);

    info := run_one(*b);
    if !info return false;

    defer free_query_result(info);

    if info.row_count == 0 {
        if !psql.quiet print_error(tprint("Did not find any relation with OID %.", oid));
        return false;
    }

    t := read_table_info(info, schemaname, relationname, oid, verbose);

    if t.relkind == Relkind.SEQUENCE {
        return describe_sequence(schemaname, relationname, oid, t.relpersistence);
    }

    b = new_sql();
    l := build_column_query(*b, *t);

    res := run_one(*b);

    if !res return false;

    defer free_query_result(res);

    cont: Content;

    content_init(*cont, table_title(*t));
    add_column_headers(*cont, *l);

    for row: 0..res.row_count - 1 add_column_row(*cont, res, row, *l);

    if !add_kind_footers(*cont, *t) return false;

    if t.hastriggers && !add_trigger_footers(*cont, oid) return false;

    if has_table_footers(t.relkind) {
        if !add_inheritance_footers(*cont, schemaname, oid, t.relkind, verbose, t.reloftype, t.relreplident, t.hasoids, t.tablespace, t.relam, t.has_relam) return false;
    }

    if verbose && t.reloptions.count > 0 {
        add_footer(*cont, tprint("Options: %", t.reloptions));
    }

    emit_content(*cont, options);

    return true;
}

run_one :: (b: *String_Builder) -> *Pg.Result {
    return psql_exec(sql_text(b));
}

add_sequence_publication_footer :: (footers: *[..] string, oid: string) -> bool {
    result := psql_exec(tprint("SELECT pubname FROM pg_catalog.pg_publication p\nWHERE p.puballsequences\n AND pg_catalog.pg_relation_is_publishable('%')\nORDER BY 1", oid));
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    text := "Included in publications:";
    for i: 0..result.row_count - 1 text = tprint("%\n    \"%\"", text, cell(result, i, 0));

    array_add(footers, text);

    return true;
}

describe_sequence :: (schemaname: string, relationname: string, oid: string, relpersistence: u8) -> bool {
    b := new_sql();

    putf(*b, "SELECT pg_catalog.format_type(seqtypid, NULL) AS \"Type\",\n       seqstart AS \"Start\",\n       seqmin AS \"Minimum\",\n       seqmax AS \"Maximum\",\n       seqincrement AS \"Increment\",\n       CASE WHEN seqcycle THEN 'yes' ELSE 'no' END AS \"Cycles?\",\n       seqcache AS \"Cache\"\nFROM pg_catalog.pg_sequence\nWHERE seqrelid = '%';", oid);

    res := run_one(*b);
    if !res return false;

    defer free_query_result(res);

    b = new_sql();
    putf(*b, "SELECT pg_catalog.quote_ident(nspname) || '.' ||\n   pg_catalog.quote_ident(relname) || '.' ||\n   pg_catalog.quote_ident(attname),\n   d.deptype\nFROM pg_catalog.pg_class c\nINNER JOIN pg_catalog.pg_depend d ON c.oid=d.refobjid\nINNER JOIN pg_catalog.pg_namespace n ON n.oid=c.relnamespace\nINNER JOIN pg_catalog.pg_attribute a ON (\n a.attrelid=c.oid AND\n a.attnum=d.refobjsubid)\nWHERE d.classid='pg_catalog.pg_class'::pg_catalog.regclass\n AND d.refclassid='pg_catalog.pg_class'::pg_catalog.regclass\n AND d.objid='%'\n AND d.deptype IN ('a', 'i')", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    footers: [..] string;
    footers.allocator = temp;

    if result.row_count == 1 {
        kind := first_char(cell(result, 0, 1));
        if kind == #char "a" array_add(*footers, tprint("Owned by: %", cell(result, 0, 0)));
        else if kind == #char "i" array_add(*footers, tprint("Sequence for identity column: %", cell(result, 0, 0)));
    }

    if at_least(190000) && !add_sequence_publication_footer(*footers, oid) return false;

    title := tprint("Sequence \"%.%\"", schemaname, relationname);
    if relpersistence == #char "u" title = tprint("Unlogged sequence \"%.%\"", schemaname, relationname);

    print_result(res, title, psql.popt, footers, false);

    return true;
}

add_tablespace_footer :: (cont: *Content, relkind: u8, tablespace: string, newline: bool) {
    if !(relkind == Relkind.RELATION || relkind == Relkind.MATVIEW || relkind == Relkind.INDEX
        || relkind == Relkind.PARTITIONED_TABLE || relkind == Relkind.PARTITIONED_INDEX || relkind == Relkind.TOASTVALUE) return;

    if to_int(tablespace) == 0 return;

    result := psql_exec(tprint("SELECT spcname FROM pg_catalog.pg_tablespace\nWHERE oid = '%';", tablespace));
    if !result return;

    defer free_query_result(result);

    if result.row_count > 0 {
        if newline add_footer(cont, tprint("Tablespace: \"%\"", cell(result, 0, 0)));
        else if cont.footers.count > 0 set_last_footer(cont, tprint("%, tablespace \"%\"", cont.footers[cont.footers.count - 1], cell(result, 0, 0)));
    }
}

// One line of the Indexes list.
index_footer_line :: (result: *Pg.Result, i: s64) -> string {
    line := tprint("    \"%\"", cell(result, i, 0));

    // An exclusion constraint, or a key over a period (WITHOUT OVERLAPS), prints its definition.
    if cell(result, i, 7) == "x" || is_true(result, i, 12) {
        line = tprint("% %", line, cell(result, i, 6));
    } else {
        if is_true(result, i, 1) line = tprint("% PRIMARY KEY,", line);
        else if is_true(result, i, 2) {
            if cell(result, i, 7) == "u" line = tprint("% UNIQUE CONSTRAINT,", line);
            else line = tprint("% UNIQUE,", line);
        }

        indexdef := cell(result, i, 5);
        using_at := find_index_from_left(indexdef, " USING ");

        if using_at >= 0 indexdef = advance(indexdef, using_at + 7);

        line = tprint("% %", line, indexdef);

        if is_true(result, i, 8) line = tprint("% DEFERRABLE", line);
        if is_true(result, i, 9) line = tprint("% INITIALLY DEFERRED", line);
    }

    if is_true(result, i, 3) line = tprint("% CLUSTER", line);
    if !is_true(result, i, 4) line = tprint("% INVALID", line);
    if is_true(result, i, 10) line = tprint("% REPLICA IDENTITY", line);

    return line;
}

add_index_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT c2.relname, i.indisprimary, i.indisunique, i.indisclustered, i.indisvalid, pg_catalog.pg_get_indexdef(i.indexrelid, 0, true),\n  pg_catalog.pg_get_constraintdef(con.oid, true), contype, condeferrable, condeferred, i.indisreplident, c2.reltablespace, %\nFROM pg_catalog.pg_class c, pg_catalog.pg_class c2, pg_catalog.pg_index i\n  LEFT JOIN pg_catalog.pg_constraint con ON (conrelid = i.indrelid AND conindid = i.indexrelid AND contype IN ('p','u','x'))\nWHERE c.oid = '%' AND c.oid = i.indrelid AND i.indexrelid = c2.oid\nORDER BY i.indisprimary DESC, c2.relname;", ifx at_least(180000) then "con.conperiod" else "false", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Indexes:");

    for i: 0..result.row_count - 1 {
        add_footer(cont, index_footer_line(result, i));
        add_tablespace_footer(cont, Relkind.INDEX, cell(result, i, 11), false);
    }

    return true;
}

add_check_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT r.conname, pg_catalog.pg_get_constraintdef(r.oid, true)\nFROM pg_catalog.pg_constraint r\nWHERE r.conrelid = '%' AND r.contype = 'c'\nORDER BY 1;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Check constraints:");
    for i: 0..result.row_count - 1 add_footer(cont, tprint("    \"%\" %", cell(result, i, 0), cell(result, i, 1)));

    return true;
}

// PostgreSQL 18 keeps NOT NULL as constraints and lists them under \d+.
add_not_null_footers :: (cont: *Content, oid: string, relkind: u8) -> bool {
    if relkind != Relkind.RELATION && relkind != Relkind.PARTITIONED_TABLE && relkind != Relkind.FOREIGN_TABLE return true;

    b := new_sql();

    putf(*b, "SELECT c.conname, a.attname, c.connoinherit,\n  c.conislocal, c.coninhcount <> 0, c.convalidated\nFROM pg_catalog.pg_constraint c JOIN\n  pg_catalog.pg_attribute a ON\n    (a.attrelid = c.conrelid AND a.attnum = c.conkey[1])\nWHERE c.contype = 'n' AND\n  c.conrelid = '%'::pg_catalog.regclass\nORDER BY a.attnum;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Not-null constraints:");

    for i: 0..result.row_count - 1 {
        line := tprint("    \"%\" NOT NULL \"%\"", cell(result, i, 0), cell(result, i, 1));
        if is_true(result, i, 2) line = tprint("% NO INHERIT", line);
        if !is_true(result, i, 5) line = tprint("% NOT VALID", line);

        local := is_true(result, i, 3);
        inherited := is_true(result, i, 4);

        if local && inherited line = tprint("% (local, inherited)", line);
        else if inherited line = tprint("% (inherited)", line);

        add_footer(cont, line);
    }

    return true;
}

add_foreign_key_footers :: (cont: *Content, oid: string, inherits_keys: bool) -> bool {
    b := new_sql();

    if inherits_keys {
        putf(*b, "SELECT conrelid = '%'::pg_catalog.regclass AS sametable,\n       conname,\n       pg_catalog.pg_get_constraintdef(oid, true) AS condef,\n       conrelid::pg_catalog.regclass AS ontable\n  FROM pg_catalog.pg_constraint,\n       pg_catalog.pg_partition_ancestors('%')\n WHERE conrelid = relid AND contype = 'f' AND conparentid = 0\nORDER BY sametable DESC, conname;", oid, oid);
    } else {
        putf(*b, "SELECT true as sametable, conname,\n  pg_catalog.pg_get_constraintdef(r.oid, true) as condef,\n  conrelid::pg_catalog.regclass AS ontable\nFROM pg_catalog.pg_constraint r\nWHERE r.conrelid = '%' AND r.contype = 'f'\n     AND conparentid = 0\nORDER BY conname", oid);
    }

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Foreign-key constraints:");

    for i: 0..result.row_count - 1 {
        if cell(result, i, 0) == "f" {
            add_footer(cont, tprint("    TABLE \"%\" CONSTRAINT \"%\" %", cell(result, i, 3), cell(result, i, 1), cell(result, i, 2)));
        } else {
            add_footer(cont, tprint("    \"%\" %", cell(result, i, 1), cell(result, i, 2)));
        }
    }

    return true;
}

add_referenced_by_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT conname, conrelid::pg_catalog.regclass AS ontable,\n       pg_catalog.pg_get_constraintdef(oid, true) AS condef\n  FROM pg_catalog.pg_constraint c\n WHERE confrelid IN (SELECT pg_catalog.pg_partition_ancestors('%')\n                     UNION ALL VALUES ('%'::pg_catalog.regclass))\n       AND contype = 'f' AND conparentid = 0\nORDER BY conname;", oid, oid);

    refs := run_one(*b);
    if !refs return false;

    defer free_query_result(refs);

    if refs.row_count == 0 return true;

    add_footer(cont, "Referenced by:");

    for i: 0..refs.row_count - 1 {
        add_footer(cont, tprint("    TABLE \"%\" CONSTRAINT \"%\" %", cell(refs, i, 1), cell(refs, i, 0), cell(refs, i, 2)));
    }

    return true;
}

// The heading above the policies, or none when there is nothing to say.
policies_heading :: (rowsecurity: bool, forcerowsecurity: bool, tuples: s64) -> string {
    if !rowsecurity return ifx tuples > 0 then "Policies (row security disabled):" else "";

    if forcerowsecurity {
        return ifx tuples > 0 then "Policies (forced row security enabled):" else "Policies (forced row security enabled): (none)";
    }

    return ifx tuples > 0 then "Policies:" else "Policies (row security enabled): (none)";
}

policy_footer_line :: (result: *Pg.Result, i: s64) -> string {
    line := tprint("    POLICY \"%\"", cell(result, i, 0));
    if first_char(cell(result, i, 1)) == #char "f" line = tprint("% AS RESTRICTIVE", line);
    if !is_null(result, i, 5) line = tprint("% FOR %", line, cell(result, i, 5));
    if !is_null(result, i, 2) line = tprint("%\n      TO %", line, cell(result, i, 2));
    if !is_null(result, i, 3) line = tprint("%\n      USING (%)", line, cell(result, i, 3));
    if !is_null(result, i, 4) line = tprint("%\n      WITH CHECK (%)", line, cell(result, i, 4));

    return line;
}

add_policy_footers :: (cont: *Content, oid: string, rowsecurity: bool, forcerowsecurity: bool) -> bool {
    b := new_sql();

    putf(*b, "SELECT pol.polname, pol.polpermissive,\n  CASE WHEN pol.polroles = '{0}' THEN NULL ELSE pg_catalog.array_to_string(array(select rolname from pg_catalog.pg_roles where oid = any (pol.polroles) order by 1),',') END,\n  pg_catalog.pg_get_expr(pol.polqual, pol.polrelid),\n  pg_catalog.pg_get_expr(pol.polwithcheck, pol.polrelid),\n  CASE pol.polcmd\n    WHEN 'r' THEN 'SELECT'\n    WHEN 'a' THEN 'INSERT'\n    WHEN 'w' THEN 'UPDATE'\n    WHEN 'd' THEN 'DELETE'\n    END AS cmd\nFROM pg_catalog.pg_policy pol\nWHERE pol.polrelid = '%' ORDER BY 1;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);
    tuples := result.row_count;

    heading := policies_heading(rowsecurity, forcerowsecurity, tuples);

    if heading.count add_footer(cont, heading);

    for i: 0..tuples - 1 add_footer(cont, policy_footer_line(result, i));

    return true;
}

statistics_footer_line :: (result: *Pg.Result, i: s64) -> string {
    has_nd := is_true(result, i, 5);
    has_dep := is_true(result, i, 6);
    has_mcv := is_true(result, i, 7);

    line := tprint("    \"%.%\"", cell(result, i, 2), cell(result, i, 3));

    if (has_nd || has_dep || has_mcv) && !(has_nd && has_dep && has_mcv) {
        kinds: [..] string;
        kinds.allocator = temp;
        if has_nd array_add(*kinds, "ndistinct");
        if has_dep array_add(*kinds, "dependencies");
        if has_mcv array_add(*kinds, "mcv");

        line = tprint("% (%)", line, join(..kinds, separator = ", ",, temp));
    }

    line = tprint("% ON % FROM %", line, cell(result, i, 4), cell(result, i, 1));

    if !is_null(result, i, 8) && cell(result, i, 8) != "-1" line = tprint("%; STATISTICS %", line, cell(result, i, 8));

    return line;
}

add_statistics_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT oid, stxrelid::pg_catalog.regclass, stxnamespace::pg_catalog.regnamespace::pg_catalog.text AS nsp, stxname,\npg_catalog.pg_get_statisticsobjdef_columns(oid) AS columns,\n  'd' = any(stxkind) AS ndist_enabled,\n  'f' = any(stxkind) AS deps_enabled,\n  'm' = any(stxkind) AS mcv_enabled,\nstxstattarget\nFROM pg_catalog.pg_statistic_ext\nWHERE stxrelid = '%'\nORDER BY nsp, stxname;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    if result.row_count == 0 return true;

    add_footer(cont, "Statistics objects:");

    for i: 0..result.row_count - 1 add_footer(cont, statistics_footer_line(result, i));

    return true;
}

add_rule_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT r.rulename, trim(trailing ';' from pg_catalog.pg_get_ruledef(r.oid, true)), ev_enabled\nFROM pg_catalog.pg_rewrite r\nWHERE r.ev_class = '%' ORDER BY 1;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    headings := string.["Rules:", "Disabled rules:", "Rules firing always:", "Rules firing on replica only:"];
    modes := string.["O", "D", "A", "R"];

    for category: 0..3 {
        have_heading := false;

        for i: 0..result.row_count - 1 {
            if first_char(cell(result, i, 2)) != modes[category][0] continue;

            if !have_heading {
                add_footer(cont, headings[category]);
                have_heading = true;
            }

            add_footer(cont, tprint("    %", advance(cell(result, i, 1), 12)));
        }
    }

    return true;
}

publication_footer_query :: (oid: string) -> string {
    tail := tprint("WHERE pr.prrelid = '%'\nUNION\nSELECT pubname\n     , NULL\n     , NULL\nFROM pg_catalog.pg_publication p\nWHERE p.puballtables AND pg_catalog.pg_relation_is_publishable('%')\nORDER BY 1;", oid, oid);

    if at_least(190000) {
        tail = tprint("WHERE pr.prrelid = '%'\n AND NOT pr.prexcept\nUNION\nSELECT pubname\n     , NULL\n     , NULL\nFROM pg_catalog.pg_publication p\nWHERE p.puballtables AND pg_catalog.pg_relation_is_publishable('%')\n     AND NOT EXISTS (\n     SELECT 1\n     FROM pg_catalog.pg_publication_rel pr\n     WHERE pr.prpubid = p.oid AND\n     (pr.prrelid = '%' OR pr.prrelid = pg_catalog.pg_partition_root('%')))\nORDER BY 1;", oid, oid, oid, oid);
    }

    return tprint("SELECT pubname\n     , NULL\n     , NULL\nFROM pg_catalog.pg_publication p\n     JOIN pg_catalog.pg_publication_namespace pn ON p.oid = pn.pnpubid\n     JOIN pg_catalog.pg_class pc ON pc.relnamespace = pn.pnnspid\nWHERE pc.oid ='%' and pg_catalog.pg_relation_is_publishable('%')\nUNION\nSELECT pubname\n     , pg_catalog.pg_get_expr(pr.prqual, c.oid)\n     , (CASE WHEN pr.prattrs IS NOT NULL THEN\n         (SELECT pg_catalog.string_agg(attname, ', ')\n           FROM pg_catalog.generate_series(0, pg_catalog.array_upper(pr.prattrs::pg_catalog.int2[], 1)) s,\n                pg_catalog.pg_attribute\n          WHERE attrelid = pr.prrelid AND attnum = prattrs[s])\n        ELSE NULL END) FROM pg_catalog.pg_publication p\n     JOIN pg_catalog.pg_publication_rel pr ON p.oid = pr.prpubid\n     JOIN pg_catalog.pg_class c ON c.oid = pr.prrelid\n%", oid, oid, tail);
}

add_publication_footers :: (cont: *Content, oid: string) -> bool {
    result := psql_exec(publication_footer_query(oid));
    if !result return false;

    defer free_query_result(result);

    if result.row_count > 0 add_footer(cont, ifx at_least(190000) then "Included in publications:" else "Publications:");

    for i: 0..result.row_count - 1 {
        line := tprint("    \"%\"", cell(result, i, 0));
        if !is_null(result, i, 2) line = tprint("% (%)", line, cell(result, i, 2));
        if !is_null(result, i, 1) line = tprint("% WHERE %", line, cell(result, i, 1));
        add_footer(cont, line);
    }

    if at_least(190000) return add_excluded_publication_footers(cont, oid);

    return true;
}

add_excluded_publication_footers :: (cont: *Content, oid: string) -> bool {
    result := psql_exec(tprint("SELECT pubname\nFROM pg_catalog.pg_publication p\nJOIN pg_catalog.pg_publication_rel pr ON p.oid = pr.prpubid\nWHERE (pr.prrelid = '%' OR pr.prrelid = pg_catalog.pg_partition_root('%'))\nAND pr.prexcept\nORDER BY 1;", oid, oid));
    if !result return false;

    defer free_query_result(result);

    if result.row_count > 0 add_footer(cont, "Excluded from publications:");

    for i: 0..result.row_count - 1 add_footer(cont, tprint("    \"%\"", cell(result, i, 0)));

    return true;
}

add_table_footers :: (cont: *Content, oid: string, relkind: u8, hasindex: bool, checks: s64, hastriggers: bool, ispartition: bool, rowsecurity: bool, forcerowsecurity: bool, hasrules: bool) -> bool {
    if hasindex && !add_index_footers(cont, oid) return false;
    if checks != 0 && !add_check_footers(cont, oid) return false;

    // PostgreSQL 18's foreign keys that are not enforced have no triggers, but a partition still
    // lists the ones it inherits.
    if hastriggers || relkind == Relkind.PARTITIONED_TABLE || (ispartition && at_least(180000)) {
        inherits_keys := ispartition || relkind == Relkind.PARTITIONED_TABLE;

        if !add_foreign_key_footers(cont, oid, inherits_keys) return false;
        if !add_referenced_by_footers(cont, oid) return false;
    }

    if !add_policy_footers(cont, oid, rowsecurity, forcerowsecurity) return false;
    if !add_statistics_footers(cont, oid) return false;
    if hasrules && relkind != Relkind.MATVIEW && !add_rule_footers(cont, oid) return false;

    return add_publication_footers(cont, oid);
}

add_trigger_footers :: (cont: *Content, oid: string) -> bool {
    b := new_sql();

    putf(*b, "SELECT t.tgname, pg_catalog.pg_get_triggerdef(t.oid, true), t.tgenabled, t.tgisinternal,\n  CASE WHEN t.tgparentid != 0 THEN\n    (SELECT u.tgrelid::pg_catalog.regclass\n     FROM pg_catalog.pg_trigger AS u,\n          pg_catalog.pg_partition_ancestors(t.tgrelid) WITH ORDINALITY AS a(relid, depth)\n     WHERE u.tgname = t.tgname AND u.tgrelid = a.relid\n           AND u.tgparentid = 0\n     ORDER BY a.depth LIMIT 1)\n  END AS parent\nFROM pg_catalog.pg_trigger t\nWHERE t.tgrelid = '%' AND (NOT t.tgisinternal OR (t.tgisinternal AND t.tgenabled = 'D'))\nORDER BY 1;", oid);

    result := run_one(*b);
    if !result return false;

    defer free_query_result(result);

    headings := string.["Triggers:", "Disabled user triggers:", "Disabled internal triggers:", "Triggers firing always:", "Triggers firing on replica only:"];

    for category: 0..4 {
        have_heading := false;

        for i: 0..result.row_count - 1 {
            enabled := first_char(cell(result, i, 2));
            internal := first_char(cell(result, i, 3));
            listed := false;

            if category == {
                case 0; listed = enabled == #char "O" || enabled == #char "t";
                case 1; listed = (enabled == #char "D" || enabled == #char "f") && internal == #char "f";
                case 2; listed = (enabled == #char "D" || enabled == #char "f") && internal == #char "t";
                case 3; listed = enabled == #char "A";
                case 4; listed = enabled == #char "R";
            }

            if !listed continue;

            if !have_heading {
                add_footer(cont, headings[category]);
                have_heading = true;
            }

            definition := cell(result, i, 1);
            at := find_index_from_left(definition, " TRIGGER ");

            if at >= 0 definition = advance(definition, at + 9);

            line := tprint("    %", definition);
            if !is_null(result, i, 4) line = tprint("%, ON TABLE %", line, cell(result, i, 4));

            add_footer(cont, line);
        }
    }

    return true;
}

add_inheritance_footers :: (cont: *Content, schemaname: string, oid: string, relkind: u8, verbose: bool, reloftype: string, relreplident: u8, hasoids: bool, tablespace: string, relam: string, has_relam: bool) -> bool {
    is_partitioned := relkind == Relkind.PARTITIONED_TABLE || relkind == Relkind.PARTITIONED_INDEX;

    if relkind == Relkind.FOREIGN_TABLE {
        sql := tprint("SELECT s.srvname,\n  pg_catalog.array_to_string(ARRAY(\n    SELECT pg_catalog.quote_ident(option_name) || ' ' || pg_catalog.quote_literal(option_value)\n    FROM pg_catalog.pg_options_to_table(ftoptions)),  ', ')\nFROM pg_catalog.pg_foreign_table f,\n     pg_catalog.pg_foreign_server s\nWHERE f.ftrelid = '%' AND s.oid = f.ftserver;", oid);
        result := psql_exec(sql);

        if !result return false;

        defer free_query_result(result);
        if result.row_count != 1 return false;

        add_footer(cont, tprint("Server: %", cell(result, 0, 0)));
        options := cell(result, 0, 1);
        if options.count add_footer(cont, tprint("FDW options: (%)", options));
    }

    {
        result := psql_exec(tprint("SELECT c.oid::pg_catalog.regclass\nFROM pg_catalog.pg_class c, pg_catalog.pg_inherits i\nWHERE c.oid = i.inhparent AND i.inhrelid = '%'\n  AND c.relkind != 'p' AND c.relkind != 'I'\nORDER BY inhseqno;", oid));
        if !result return false;

        defer free_query_result(result);

        if at_least(190000) {
            if result.row_count > 0 add_footer(cont, "Inherits:");

            for i: 0..result.row_count - 1 add_footer(cont, tprint("    %", cell(result, i, 0)));
        } else {
            for i: 0..result.row_count - 1 {
                line: string;
                if i == 0 line = tprint("Inherits: %", cell(result, i, 0));
                else line = tprint("          %", cell(result, i, 0));
                if i < result.row_count - 1 line = tprint("%,", line);

                add_footer(cont, line);
            }
        }
    }

    {
        result := psql_exec(tprint("SELECT c.oid::pg_catalog.regclass, c.relkind, inhdetachpending, pg_catalog.pg_get_expr(c.relpartbound, c.oid)\nFROM pg_catalog.pg_class c, pg_catalog.pg_inherits i\nWHERE c.oid = i.inhrelid AND i.inhparent = '%'\nORDER BY pg_catalog.pg_get_expr(c.relpartbound, c.oid) = 'DEFAULT', c.oid::pg_catalog.regclass::pg_catalog.text;", oid));
        if !result return false;

        defer free_query_result(result);
        tuples := result.row_count;

        if is_partitioned && tuples == 0 {
            add_footer(cont, "Number of partitions: 0");
        } else if !verbose {
            if tuples > 0 {
                if is_partitioned add_footer(cont, tprint("Number of partitions: % (Use \\d+ to list them.)", tuples));
                else add_footer(cont, tprint("Number of child tables: % (Use \\d+ to list them.)", tuples));
            }
        } else {
            label := ifx is_partitioned then "Partitions" else "Child tables";
            listed := at_least(190000);
            indent := copy_string(label,, temp);

            for 0..indent.count - 1 indent.data[it] = #char " ";

            if listed && tuples > 0 add_footer(cont, tprint("%:", label));

            for i: 0..tuples - 1 {
                child := first_char(cell(result, i, 1));
                line: string;

                if listed line = tprint("    %", cell(result, i, 0));
                else if i == 0 line = tprint("%: %", label, cell(result, i, 0));
                else line = tprint("%  %", indent, cell(result, i, 0));

                if !is_null(result, i, 3) line = tprint("% %", line, cell(result, i, 3));

                if child == Relkind.PARTITIONED_TABLE || child == Relkind.PARTITIONED_INDEX line = tprint("%, PARTITIONED", line);
                else if child == Relkind.FOREIGN_TABLE && at_least(160000) line = tprint("%, FOREIGN", line);

                if cell(result, i, 2) == "t" line = tprint("% (DETACH PENDING)", line);
                if !listed && i < tuples - 1 line = tprint("%,", line);

                add_footer(cont, line);
            }
        }
    }

    if reloftype.count add_footer(cont, tprint("Typed table of type: %", reloftype));

    if verbose && (relkind == Relkind.RELATION || relkind == Relkind.MATVIEW) && relreplident != #char "i"
        && ((schemaname != "pg_catalog" && relreplident != #char "d") || (schemaname == "pg_catalog" && relreplident != #char "n")) {
        name := "???";
        if relreplident == #char "f" name = "FULL";
        else if relreplident == #char "n" name = "NOTHING";
        add_footer(cont, tprint("Replica Identity: %", name));
    }

    if verbose && relkind != Relkind.MATVIEW && hasoids add_footer(cont, "Has OIDs: yes");

    add_tablespace_footer(cont, relkind, tablespace, true);

    if verbose && has_relam && !psql.hide_tableam add_footer(cont, tprint("Access method: %", relam));

    return true;
}

// \dt \dv \dm \di \ds \dE and \d without a name

list_tables :: (tabtypes: string, pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    show_tables := contains(tabtypes, "t");
    show_indexes := contains(tabtypes, "i");
    show_views := contains(tabtypes, "v");
    show_matviews := contains(tabtypes, "m");
    show_seq := contains(tabtypes, "s");
    show_foreign := contains(tabtypes, "E");

    if !(show_tables || show_indexes || show_views || show_matviews || show_seq || show_foreign) {
        show_tables = true;
        show_views = true;
        show_matviews = true;
        show_seq = true;
        show_foreign = true;
    }

    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  c.relname as \"Name\",\n  CASE c.relkind WHEN 'r' THEN 'table' WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized view' WHEN 'i' THEN 'index' WHEN 'S' THEN 'sequence' WHEN 't' THEN 'TOAST table' WHEN 'f' THEN 'foreign table' WHEN 'p' THEN 'partitioned table' WHEN 'I' THEN 'partitioned index' END as \"Type\",\n  pg_catalog.pg_get_userbyid(c.relowner) as \"Owner\"");

    if show_indexes put(*b, ",\n  c2.relname as \"Table\"");

    with_am := !psql.hide_tableam && (show_tables || show_matviews || show_indexes);

    if verbose {
        put(*b, ",\n  CASE c.relpersistence WHEN 'p' THEN 'permanent' WHEN 't' THEN 'temporary' WHEN 'u' THEN 'unlogged' END as \"Persistence\"");
        if with_am put(*b, ",\n  am.amname as \"Access method\"");
        put(*b, ",\n  pg_catalog.pg_size_pretty(pg_catalog.pg_table_size(c.oid)) as \"Size\",\n  pg_catalog.obj_description(c.oid, 'pg_class') as \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_class c\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace");
    if with_am put(*b, "\n     LEFT JOIN pg_catalog.pg_am am ON am.oid = c.relam");
    if show_indexes put(*b, "\n     LEFT JOIN pg_catalog.pg_index i ON i.indexrelid = c.oid\n     LEFT JOIN pg_catalog.pg_class c2 ON i.indrelid = c2.oid");

    put(*b, "\nWHERE c.relkind IN (");

    if show_tables {
        put(*b, "'r','p',");
        if show_system || pattern.set put(*b, "'t',");
    }

    if show_views put(*b, "'v',");
    if show_matviews put(*b, "'m',");
    if show_indexes put(*b, "'i','I',");
    if show_seq put(*b, "'S',");
    if show_system || pattern.set put(*b, "'s',");
    if show_foreign put(*b, "'f',");

    put(*b, "'')\n");

    if !show_system && !pattern.set {
        put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname !~ '^pg_toast'\n      AND n.nspname <> 'information_schema'\n");
    }

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.relname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 1,2;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 && !psql.quiet {
        if pattern.set print_error(tprint("Did not find any relation named \"%\".", pattern.text));
        else print_error("Did not find any relations.");
    } else {
        print_result(r, relations_title(show_tables, show_views, show_matviews, show_indexes, show_seq, show_foreign), psql.popt);
    }

    return true;
}

list_partitioned_tables :: (reltypes: string, pattern: Pattern, verbose: bool) -> bool {
    show_tables := contains(reltypes, "t");
    show_indexes := contains(reltypes, "i");
    show_nested := contains(reltypes, "n");
    mixed := false;

    if !show_tables && !show_indexes {
        show_tables = true;
        show_indexes = true;
    }

    title: string;
    if show_indexes && !show_tables title = "List of partitioned indexes";
    else if show_tables && !show_indexes title = "List of partitioned tables";
    else {
        title = "List of partitioned relations";
        mixed = true;
    }

    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  c.relname as \"Name\",\n  pg_catalog.pg_get_userbyid(c.relowner) as \"Owner\"");

    if mixed put(*b, ",\n  CASE c.relkind WHEN 'p' THEN 'partitioned table' WHEN 'I' THEN 'partitioned index' END as \"Type\"");
    if show_nested || pattern.set put(*b, ",\n  inh.inhparent::pg_catalog.regclass as \"Parent name\"");
    if show_indexes put(*b, ",\n c2.oid::pg_catalog.regclass as \"Table\"");

    if verbose {
        if show_nested put(*b, ",\n  s.dps as \"Leaf partition size\",\n  s.tps as \"Total size\"");
        else put(*b, ",\n  s.tps as \"Total size\"");
        put(*b, ",\n  pg_catalog.obj_description(c.oid, 'pg_class') as \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_class c\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace");
    if show_indexes put(*b, "\n     LEFT JOIN pg_catalog.pg_index i ON i.indexrelid = c.oid\n     LEFT JOIN pg_catalog.pg_class c2 ON i.indrelid = c2.oid");
    if show_nested || pattern.set put(*b, "\n     LEFT JOIN pg_catalog.pg_inherits inh ON c.oid = inh.inhrelid");

    if verbose {
        put(*b, ",\n     LATERAL (SELECT pg_catalog.pg_size_pretty(sum(\n                 CASE WHEN ppt.isleaf AND ppt.level = 1\n                      THEN pg_catalog.pg_table_size(ppt.relid) ELSE 0 END)) AS dps,\n                     pg_catalog.pg_size_pretty(sum(pg_catalog.pg_table_size(ppt.relid))) AS tps\n              FROM pg_catalog.pg_partition_tree(c.oid) ppt) s");
    }

    put(*b, "\nWHERE c.relkind IN (");
    if show_tables put(*b, "'p',");
    if show_indexes put(*b, "'I',");

    put(*b, "'')\n");

    if !show_nested && !pattern.set put(*b, " AND NOT c.relispartition\n");

    if !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname !~ '^pg_toast'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.relname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY \"Schema\", ");
    if mixed put(*b, "\"Type\" DESC, ");
    if show_nested || pattern.set put(*b, "\"Parent name\" NULLS FIRST, ");

    put(*b, "\"Name\";");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, title, psql.popt);

    return true;
}

// The argument-type patterns that follow the name pattern of \df and \do.
more_arguments :: (s: *Scanner, pattern: Pattern) -> [] string {
    args: [..] string;
    args.allocator = temp;

    if pattern.set {
        while args.count < 100 {
            arg, found := next_arg(s, .NORMAL, true);
            if !found break;
            array_add(*args, arg);
        }
    }

    return args;
}

// The \d family. `handled` is false for a command psql does not know.
describe_d :: (s: *Scanner, name: string, pattern: Pattern, verbose: bool, show_system: bool) -> ok: bool, handled: bool {
    sub := ifx name.count > 1 then name[1] else 0;
    third := ifx name.count > 2 then name[2] else 0;

    if sub == 0 || sub == #char "+" || sub == #char "S" {
        if pattern.set return describe_table_details(pattern, verbose, show_system), true;
        return list_tables("tvmsE", Pattern.{}, verbose, show_system), true;
    }

    if sub == #char "t" || sub == #char "v" || sub == #char "m" || sub == #char "i" || sub == #char "s" || sub == #char "E" {
        return list_tables(slice(name, 1, name.count - 1), pattern, verbose, show_system), true;
    }

    if sub == #char "P" {
        if third == 0 || third == #char "+" || third == #char "t" || third == #char "i" || third == #char "n" {
            return list_partitioned_tables(slice(name, 2, name.count - 2), pattern, verbose), true;
        }

        return false, false;
    }

    if sub == #char "a" return describe_aggregates(pattern, verbose, show_system), true;
    if sub == #char "b" return describe_tablespaces(pattern, verbose), true;
    if sub == #char "T" return describe_types(pattern, verbose, show_system), true;

    if sub == #char "f" {
        if third == 0 || third == #char "+" || third == #char "S" || third == #char "a" || third == #char "n" || third == #char "p" || third == #char "t" || third == #char "w" {
            return describe_functions(slice(name, 2, name.count - 2), pattern, more_arguments(s, pattern), verbose, show_system), true;
        }

        return false, false;
    }

    if sub == #char "o" return describe_operators(pattern, more_arguments(s, pattern), verbose, show_system), true;

    if sub == #char "A" {
        pattern2 := Pattern.{};

        if pattern.set && third != 0 && third != #char "+" {
            arg, found := next_arg(s, .NORMAL, true);
            pattern2 = .{ arg, found };
        }

        if third == 0 || third == #char "+" return describe_access_methods(pattern, verbose), true;
        if third == #char "c" return list_operator_classes(pattern, pattern2, verbose), true;
        if third == #char "f" return list_operator_families(pattern, pattern2, verbose), true;
        if third == #char "o" return list_opfamily_operators(pattern, pattern2, verbose), true;
        if third == #char "p" return list_opfamily_functions(pattern, pattern2, verbose), true;

        return false, false;
    }

    if sub == #char "u" || sub == #char "g" return describe_roles(pattern, verbose, show_system), true;
    if sub == #char "D" return list_domains(pattern, verbose, show_system), true;
    if sub == #char "L" return list_languages(pattern, verbose, show_system), true;
    if sub == #char "n" return list_schemas(pattern, verbose, show_system), true;
    if sub == #char "O" return list_collations(pattern, verbose, show_system), true;
    if sub == #char "C" return list_casts(pattern, verbose), true;
    if sub == #char "X" return list_extended_stats(pattern, verbose), true;
    if sub == #char "y" return list_event_triggers(pattern, verbose), true;

    if sub == #char "c" {
        if begins_with(name, "dconfig") return describe_configuration_parameters(pattern, verbose, show_system), true;
        return list_conversions(pattern, verbose, show_system), true;
    }

    if sub == #char "r" {
        if third == #char "d" && name.count > 3 && name[3] == #char "s" {
            pattern2 := Pattern.{};

            if pattern.set {
                arg, found := next_arg(s, .NORMAL, true);
                pattern2 = .{ arg, found };
            }

            return list_db_role_settings(pattern, pattern2), true;
        }

        if third == #char "g" return describe_role_grants(pattern, show_system), true;

        return false, false;
    }

    if sub == #char "x" return ifx verbose then list_extension_contents(pattern) else list_extensions(pattern), true;

    if sub == #char "F" {
        if third == 0 || third == #char "+" return list_ts_configs(pattern, verbose), true;
        if third == #char "p" return list_ts_parsers(pattern, verbose), true;
        if third == #char "d" return list_ts_dictionaries(pattern, verbose), true;
        if third == #char "t" return list_ts_templates(pattern, verbose), true;

        return false, false;
    }

    if sub == #char "e" {
        if third == #char "s" return list_foreign_servers(pattern, verbose), true;
        if third == #char "u" return list_user_mappings(pattern, verbose), true;
        if third == #char "w" return list_foreign_data_wrappers(pattern, verbose), true;
        if third == #char "t" return list_foreign_tables(pattern, verbose), true;

        return false, false;
    }

    if sub == #char "p" return permissions_list(pattern, show_system), true;
    if sub == #char "l" return list_large_objects(verbose), true;

    if sub == #char "d" {
        if begins_with(name, "ddp") return list_default_acls(pattern), true;
        return object_description(pattern, show_system), true;
    }

    if sub == #char "R" {
        if third == #char "p" return ifx verbose then describe_publications(pattern) else list_publications(pattern), true;
        if third == #char "s" return describe_subscriptions(pattern, verbose), true;
        return false, false;
    }

    return false, false;
}

// \l, \z, \ddp, \dd, \dl

acl_column :: (b: *String_Builder, column: string) {
    putf(b, "CASE WHEN pg_catalog.array_length(%, 1) = 0 THEN '(none)' ELSE pg_catalog.array_to_string(%, E'\\n') END AS \"Access privileges\"", column, column);
}

list_all_dbs :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  d.datname as \"Name\",\n  pg_catalog.pg_get_userbyid(d.datdba) as \"Owner\",\n  pg_catalog.pg_encoding_to_char(d.encoding) as \"Encoding\",\n");
    put(*b, "  CASE d.datlocprovider WHEN 'b' THEN 'builtin' WHEN 'c' THEN 'libc' WHEN 'i' THEN 'icu' END AS \"Locale Provider\",\n  d.datcollate as \"Collate\",\n  d.datctype as \"Ctype\",\n");

    if at_least(170000) put(*b, "  d.datlocale as \"Locale\",\n");
    else put(*b, "  d.daticulocale as \"Locale\",\n");

    if at_least(160000) put(*b, "  d.daticurules as \"ICU Rules\",\n");
    else put(*b, "  NULL as \"ICU Rules\",\n");

    put(*b, "  ");
    acl_column(*b, "d.datacl");

    if verbose {
        put(*b, ",\n  CASE WHEN pg_catalog.has_database_privilege(d.datname, 'CONNECT')\n       OR pg_catalog.pg_has_role('pg_read_all_stats', 'USAGE')\n       THEN pg_catalog.pg_size_pretty(pg_catalog.pg_database_size(d.datname))\n       ELSE 'No Access'\n  END as \"Size\",\n  t.spcname as \"Tablespace\",\n  pg_catalog.shobj_description(d.oid, 'pg_database') as \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_database d\n");
    if verbose put(*b, "  JOIN pg_catalog.pg_tablespace t on d.dattablespace = t.oid\n");

    if pattern.set {
        if !validate_pattern(*b, pattern, false, false, "", "d.datname", "", "", 1) return false;
    }

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "List of databases", psql.popt);

    return true;
}

permissions_list :: (pattern: Pattern, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  c.relname as \"Name\",\n  CASE c.relkind WHEN 'r' THEN 'table' WHEN 'v' THEN 'view' WHEN 'm' THEN 'materialized view' WHEN 'S' THEN 'sequence' WHEN 'f' THEN 'foreign table' WHEN 'p' THEN 'partitioned table' END as \"Type\",\n  ");
    acl_column(*b, "c.relacl");

    put(*b, ",\n  pg_catalog.array_to_string(ARRAY(\n    SELECT attname || E':\\n  ' || pg_catalog.array_to_string(attacl, E'\\n  ')\n    FROM pg_catalog.pg_attribute a\n    WHERE attrelid = c.oid AND NOT attisdropped AND attacl IS NOT NULL\n  ), E'\\n') AS \"Column privileges\"");

    put(*b, ",\n  pg_catalog.array_to_string(ARRAY(\n    SELECT polname\n    || CASE WHEN NOT polpermissive THEN\n       E' (RESTRICTIVE)'\n       ELSE '' END\n    || CASE WHEN polcmd != '*' THEN\n           E' (' || polcmd::pg_catalog.text || E'):'\n       ELSE E':'\n       END\n    || CASE WHEN polqual IS NOT NULL THEN\n           E'\\n  (u): ' || pg_catalog.pg_get_expr(polqual, polrelid)\n       ELSE E''\n       END\n    || CASE WHEN polwithcheck IS NOT NULL THEN\n           E'\\n  (c): ' || pg_catalog.pg_get_expr(polwithcheck, polrelid)\n       ELSE E''\n       END    || CASE WHEN polroles <> '{0}' THEN\n           E'\\n  to: ' || pg_catalog.array_to_string(\n               ARRAY(\n                   SELECT rolname\n                   FROM pg_catalog.pg_roles\n                   WHERE oid = ANY (polroles)\n                   ORDER BY 1\n               ), E', ')\n       ELSE E''\n       END\n    FROM pg_catalog.pg_policy pol\n    WHERE polrelid = c.oid), E'\\n')\n    AS \"Policies\"");

    put(*b, "\nFROM pg_catalog.pg_class c\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\nWHERE c.relkind IN ('r','v','m','S','f','p')\n");

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.relname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "Access privileges", psql.popt);

    return true;
}

list_default_acls :: (pattern: Pattern) -> bool {
    b := new_sql();

    put(*b, "SELECT pg_catalog.pg_get_userbyid(d.defaclrole) AS \"Owner\",\n  n.nspname AS \"Schema\",\n  CASE d.defaclobjtype WHEN 'r' THEN 'table' WHEN 'S' THEN 'sequence' WHEN 'f' THEN 'function' WHEN 'T' THEN 'type' WHEN 'n' THEN 'schema' END AS \"Type\",\n  ");
    acl_column(*b, "d.defaclacl");
    put(*b, "\nFROM pg_catalog.pg_default_acl d\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = d.defaclnamespace\n");

    if !validate_pattern(*b, pattern, false, false, "", "n.nspname", "pg_catalog.pg_get_userbyid(d.defaclrole)", "", 3) return false;

    put(*b, "ORDER BY 1, 2, 3;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "Default access privileges", psql.popt);

    return true;
}

object_description :: (pattern: Pattern, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT DISTINCT tt.nspname AS \"Schema\", tt.name AS \"Name\", tt.object AS \"Object\", d.description AS \"Description\"\nFROM (\n");

    system_filter :: "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n";
    system_where :: "WHERE n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n";

    put(*b, "  SELECT pgc.oid as oid, pgc.tableoid AS tableoid,\n  n.nspname as nspname,\n  CAST(pgc.conname AS pg_catalog.text) as name,  CAST('table constraint' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_constraint pgc\n    JOIN pg_catalog.pg_class c ON c.oid = pgc.conrelid\n    LEFT JOIN pg_catalog.pg_namespace n     ON n.oid = c.relnamespace\n");
    if !show_system && !pattern.set put(*b, system_where);
    if !validate_pattern(*b, pattern, !show_system && !pattern.set, false, "n.nspname", "pgc.conname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "UNION ALL\n  SELECT pgc.oid as oid, pgc.tableoid AS tableoid,\n  n.nspname as nspname,\n  CAST(pgc.conname AS pg_catalog.text) as name,  CAST('domain constraint' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_constraint pgc\n    JOIN pg_catalog.pg_type t ON t.oid = pgc.contypid\n    LEFT JOIN pg_catalog.pg_namespace n     ON n.oid = t.typnamespace\n");
    if !show_system && !pattern.set put(*b, system_where);
    if !validate_pattern(*b, pattern, !show_system && !pattern.set, false, "n.nspname", "pgc.conname", "", "pg_catalog.pg_type_is_visible(t.oid)", 3) return false;

    put(*b, "UNION ALL\n  SELECT o.oid as oid, o.tableoid as tableoid,\n  n.nspname as nspname,\n  CAST(o.opcname AS pg_catalog.text) as name,\n  CAST('operator class' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_opclass o\n    JOIN pg_catalog.pg_am am ON o.opcmethod = am.oid\n    JOIN pg_catalog.pg_namespace n ON n.oid = o.opcnamespace\n");
    if !show_system && !pattern.set put(*b, system_filter);
    if !validate_pattern(*b, pattern, true, false, "n.nspname", "o.opcname", "", "pg_catalog.pg_opclass_is_visible(o.oid)", 3) return false;

    put(*b, "UNION ALL\n  SELECT opf.oid as oid, opf.tableoid as tableoid,\n  n.nspname as nspname,\n  CAST(opf.opfname AS pg_catalog.text) AS name,\n  CAST('operator family' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_opfamily opf\n    JOIN pg_catalog.pg_am am ON opf.opfmethod = am.oid\n    JOIN pg_catalog.pg_namespace n ON opf.opfnamespace = n.oid\n");
    if !show_system && !pattern.set put(*b, system_filter);
    if !validate_pattern(*b, pattern, true, false, "n.nspname", "opf.opfname", "", "pg_catalog.pg_opfamily_is_visible(opf.oid)", 3) return false;

    put(*b, "UNION ALL\n  SELECT r.oid as oid, r.tableoid as tableoid,\n  n.nspname as nspname,\n  CAST(r.rulename AS pg_catalog.text) as name,  CAST('rule' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_rewrite r\n       JOIN pg_catalog.pg_class c ON c.oid = r.ev_class\n       LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\n  WHERE r.rulename != '_RETURN'\n");
    if !show_system && !pattern.set put(*b, system_filter);
    if !validate_pattern(*b, pattern, true, false, "n.nspname", "r.rulename", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "UNION ALL\n  SELECT t.oid as oid, t.tableoid as tableoid,\n  n.nspname as nspname,\n  CAST(t.tgname AS pg_catalog.text) as name,  CAST('trigger' AS pg_catalog.text) as object\n  FROM pg_catalog.pg_trigger t\n       JOIN pg_catalog.pg_class c ON c.oid = t.tgrelid\n       LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\n");
    if !show_system && !pattern.set put(*b, system_where);
    if !validate_pattern(*b, pattern, !show_system && !pattern.set, false, "n.nspname", "t.tgname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, ") AS tt\n  JOIN pg_catalog.pg_description d ON (tt.oid = d.objoid AND tt.tableoid = d.classoid AND d.objsubid = 0)\nORDER BY 1, 2, 3;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "Object descriptions", psql.popt);

    return true;
}

list_large_objects :: (verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT oid as \"ID\",\n  pg_catalog.pg_get_userbyid(lomowner) as \"Owner\",\n  ");

    if verbose {
        acl_column(*b, "lomacl");
        put(*b, ",\n  ");
    }

    put(*b, "pg_catalog.obj_description(oid, 'pg_largeobject') as \"Description\"\nFROM pg_catalog.pg_largeobject_metadata\nORDER BY oid");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "Large objects", psql.popt);

    return true;
}

// \sf and \sv

// Like psql_exec, but a failure prints only its severity and primary message.
run_minimal :: (sql: string) -> *Pg.Result {
    Exec_Hooks :: struct {
        #as using hooks: Pg.Script_Hooks;
        result: *Pg.Result;
    }

    hooks: Exec_Hooks;

    hooks.on_result = (h: *Pg.Script_Hooks, result: *Pg.Result) {
        e := cast(*Exec_Hooks) h;
        if e.result free_query_result(e.result);

        e.result = New(Pg.Result);
        e.result.* = result.*;
        result.* = .{};
    };

    if !psql.db {
        print_error("You are currently not connected to a database.");
        return null;
    }

    error := Pg.run_script(psql.db, sql, *hooks);
    if error.kind != .NONE {
        severity := ifx error.kind == .SERVER && error.severity.count then error.severity else "ERROR";
        message := ifx error.message.count then error.message else "(not available)";

        print_error(tprint("%:  %", severity, message));
        Pg.free_error(*error);
        if hooks.result free_query_result(hooks.result);

        return null;
    }

    return hooks.result;
}

show_source :: (l: *Loop, name: string) -> Command_Status {
    is_func := begins_with(name, "sf");
    numbered := contains(name, "+");

    desc, found := next_arg(*l.scanner, .WHOLE_LINE, true);

    if !found {
        print_error(ifx is_func then "function name is required" else "view name is required");
        return .ERROR;
    }

    b := new_sql();

    put(*b, "SELECT ");
    put(*b, escape_literal(desc));

    if is_func {
        putf(*b, "::pg_catalog.%::pg_catalog.oid", ifx contains(desc, "(") then "regprocedure" else "regproc");
    } else {
        put(*b, "::pg_catalog.regclass::pg_catalog.oid");
    }

    r := run_minimal(sql_text(*b));
    if !r return .ERROR;

    defer free_query_result(r);

    if r.row_count != 1 {
        print_error("ERROR:  (not available)");
        return .ERROR;
    }

    oid := cell(r, 0, 0);
    text: string;

    if is_func {
        q := run_minimal(tprint("SELECT pg_catalog.pg_get_functiondef(%)", oid));
        if !q return .ERROR;

        defer free_query_result(q);

        if q.row_count != 1 {
            print_error("ERROR:  (not available)");
            return .ERROR;
        }

        text = copy_string(cell(q, 0, 0),, temp);
    } else {
        q := run_minimal(tprint("SELECT nspname, relname, relkind, pg_catalog.pg_get_viewdef(c.oid, true), (SELECT pg_catalog.string_agg(pg_catalog.quote_ident(split_part(o, '=', 1)) || '=' || CASE WHEN pg_catalog.quote_ident(substr(o, strpos(o, '=') + 1)) = substr(o, strpos(o, '=') + 1) THEN substr(o, strpos(o, '=') + 1) ELSE pg_catalog.quote_literal(substr(o, strpos(o, '=') + 1)) END, ', ') FROM pg_catalog.unnest(pg_catalog.array_remove(pg_catalog.array_remove(c.reloptions,'check_option=local'),'check_option=cascaded')) o) AS reloptions, CASE WHEN 'check_option=local' = ANY (c.reloptions) THEN 'LOCAL'::text WHEN 'check_option=cascaded' = ANY (c.reloptions) THEN 'CASCADED'::text ELSE NULL END AS checkoption, pg_catalog.quote_ident(nspname), pg_catalog.quote_ident(relname) FROM pg_catalog.pg_class c LEFT JOIN pg_catalog.pg_namespace n ON c.relnamespace = n.oid WHERE c.oid = %", oid));
        if !q return .ERROR;

        defer free_query_result(q);

        if q.row_count != 1 {
            print_error("ERROR:  (not available)");
            return .ERROR;
        }

        out := new_sql();
        ok := true;

        if first_char(cell(q, 0, 2)) == Relkind.VIEW {
            put(*out, "CREATE OR REPLACE VIEW ");
        } else {
            print_error(tprint("\"%.%\" is not a view", cell(q, 0, 0), cell(q, 0, 1)));
            ok = false;
        }

        putf(*out, "%.%", cell(q, 0, 6), cell(q, 0, 7));

        if !is_null(q, 0, 4) && cell(q, 0, 4).count > 0 putf(*out, "\n WITH (%)", cell(q, 0, 4));

        view_def := cell(q, 0, 3);

        putf(*out, " AS\n%", view_def);
        text = sql_text(*out);

        if text.count > 0 && text[text.count - 1] == #char ";" text.count -= 1;
        if cell(q, 0, 5).count > 0 text = tprint("%\n WITH % CHECK OPTION", text, cell(q, 0, 5));

        if !ok return .ERROR;
    }

    if text.count > 0 && text[text.count - 1] != #char "\n" text = tprint("%\n", text);

    if !numbered {
        write_out(text);
        return .SKIP_LINE;
    }

    in_header := is_func;
    lineno := 0;
    rest := text;

    while rest.count > 0 {
        eol := find_index_from_left(rest, #char "\n");
        line := ifx eol >= 0 then slice(rest, 0, eol) else rest;

        if in_header && (begins_with(rest, "AS ") || begins_with(rest, "BEGIN ") || begins_with(rest, "RETURN ")) in_header = false;
        if !in_header lineno += 1;

        if in_header {
            write_out(tprint("        %\n", line));
        } else {
            number := tprint("%", lineno);
            while number.count < 7 number = tprint("% ", number);
            write_out(tprint("% %\n", number, line));
        }

        if eol < 0 break;

        rest = advance(rest, eol + 1);
    }

    return .SKIP_LINE;
}

// \dRp \dRp+ \dRs

list_publications :: (pattern: Pattern) -> bool {
    b := new_sql();

    put(*b, "SELECT pubname AS \"Name\",\n  pg_catalog.pg_get_userbyid(pubowner) AS \"Owner\",\n  puballtables AS \"All tables\"");
    if at_least(190000) put(*b, ",\n  puballsequences AS \"All sequences\"");

    put(*b, ",\n  pubinsert AS \"Inserts\",\n  pubupdate AS \"Updates\",\n  pubdelete AS \"Deletes\",\n  pubtruncate AS \"Truncates\"");
    if at_least(180000) put(*b, ",\n  (CASE pubgencols WHEN 'n' THEN 'none' WHEN 's' THEN 'stored' END) AS \"Generated columns\"");

    put(*b, ",\n  pubviaroot AS \"Via root\"\nFROM pg_catalog.pg_publication\n");

    if !validate_pattern(*b, pattern, false, false, "", "pubname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "List of publications", psql.popt);

    return true;
}

publication_footer :: (cont: *Content, sql: string, heading: string, as_schema: bool) -> bool {
    r := psql_exec(sql);
    if !r return false;

    defer free_query_result(r);

    if r.row_count > 0 add_footer(cont, heading);

    for i: 0..r.row_count - 1 {
        if as_schema {
            add_footer(cont, tprint("    \"%\"", cell(r, i, 0)));
        } else {
            line := tprint("    \"%.%\"", cell(r, i, 0), cell(r, i, 1));
            if !is_null(r, i, 3) line = tprint("% (%)", line, cell(r, i, 3));
            if !is_null(r, i, 2) line = tprint("% WHERE %", line, cell(r, i, 2));
            add_footer(cont, line);
        }
    }

    return true;
}

describe_publications :: (pattern: Pattern) -> bool {
    b := new_sql();

    put(*b, "SELECT oid, pubname,\n  pg_catalog.pg_get_userbyid(pubowner) AS owner,\n  puballtables");
    if at_least(190000) put(*b, ", puballsequences");

    put(*b, ", pubinsert, pubupdate, pubdelete, pubtruncate");
    if at_least(180000) put(*b, ", (CASE pubgencols WHEN 'n' THEN 'none' WHEN 's' THEN 'stored' END)");

    put(*b, ", pubviaroot");
    if at_least(190000) put(*b, ", pg_catalog.obj_description(oid, 'pg_publication')");

    put(*b, "\nFROM pg_catalog.pg_publication\n");

    if !validate_pattern(*b, pattern, false, false, "", "pubname", "", "", 1) return false;

    put(*b, "ORDER BY 2;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 {
        if !psql.quiet {
            if pattern.set print_error(tprint("Did not find any publication named \"%\".", pattern.text));
            else print_error("Did not find any publications.");
        }

        return false;
    }

    for i: 0..r.row_count - 1 {
        pubid := cell(r, i, 0);

        cont: Content;

        content_init(*cont, tprint("Publication %", cell(r, i, 1)));
        cont.default_footer = true;

        add_header(*cont, "Owner");
        add_header(*cont, "All tables");
        if at_least(190000) add_header(*cont, "All sequences");

        for name: string.["Inserts", "Updates", "Deletes", "Truncates"] add_header(*cont, name);
        if at_least(180000) add_header(*cont, "Generated columns");

        add_header(*cont, "Via root");
        if at_least(190000) add_header(*cont, "Description");
        for col: 2..r.columns.count - 1 add_cell(*cont, cell(r, i, col));

        if cell(r, i, 3) != "t" {
            sql := tprint("SELECT n.nspname, c.relname, pg_get_expr(pr.prqual, c.oid), (CASE WHEN pr.prattrs IS NOT NULL THEN\n     pg_catalog.array_to_string(      ARRAY(SELECT attname\n              FROM\n                pg_catalog.generate_series(0, pg_catalog.array_upper(pr.prattrs::pg_catalog.int2[], 1)) s,\n                pg_catalog.pg_attribute\n        WHERE attrelid = c.oid AND attnum = prattrs[s]), ', ')\n       ELSE NULL END)\nFROM pg_catalog.pg_class c,\n     pg_catalog.pg_namespace n,\n     pg_catalog.pg_publication_rel pr\nWHERE c.relnamespace = n.oid\n  AND c.oid = pr.prrelid\n  AND pr.prpubid = '%'\n%ORDER BY 1,2", pubid, ifx at_least(190000) then "  AND NOT pr.prexcept\n" else "");
            if !publication_footer(*cont, sql, "Tables:", false) return false;

            sql = tprint("SELECT n.nspname\nFROM pg_catalog.pg_namespace n\n     JOIN pg_catalog.pg_publication_namespace pn ON n.oid = pn.pnnspid\nWHERE pn.pnpubid = '%'\nORDER BY 1", pubid);
            if !publication_footer(*cont, sql, "Tables from schemas:", true) return false;
        } else if at_least(190000) {
            sql := tprint("SELECT n.nspname || '.' || c.relname\nFROM pg_catalog.pg_class c\n     JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\n     JOIN pg_catalog.pg_publication_rel pr ON c.oid = pr.prrelid\nWHERE pr.prpubid = '%' AND pr.prexcept\nORDER BY 1", pubid);
            if !publication_footer(*cont, sql, "Except tables:", true) return false;
        }

        emit_content(*cont, psql.popt);
    }

    return true;
}

describe_subscriptions :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT subname AS \"Name\"\n,  pg_catalog.pg_get_userbyid(subowner) AS \"Owner\"\n,  subenabled AS \"Enabled\"\n,  subpublications AS \"Publication\"\n");

    if verbose {
        put(*b, ", subbinary AS \"Binary\"\n");

        if at_least(160000) put(*b, ", (CASE substream\n    WHEN 'f' THEN 'off'\n    WHEN 't' THEN 'on'\n    WHEN 'p' THEN 'parallel'\n   END) AS \"Streaming\"\n");
        else put(*b, ", substream AS \"Streaming\"\n");

        put(*b, ", subtwophasestate AS \"Two-phase commit\"\n, subdisableonerr AS \"Disable on error\"\n");
        if at_least(160000) put(*b, ", suborigin AS \"Origin\"\n, subpasswordrequired AS \"Password required\"\n, subrunasowner AS \"Run as owner?\"\n");
        if at_least(170000) put(*b, ", subfailover AS \"Failover\"\n");
        if at_least(190000) put(*b, ", (select srvname from pg_catalog.pg_foreign_server where oid=subserver) AS \"Server\"\n, subretaindeadtuples AS \"Retain dead tuples\"\n, submaxretention AS \"Max retention duration\"\n, subretentionactive AS \"Retention active\"\n");

        put(*b, ",  subsynccommit AS \"Synchronous commit\"\n,  subconninfo AS \"Conninfo\"\n");
        if at_least(190000) put(*b, ", subwalrcvtimeout AS \"Receiver timeout\"\n");

        put(*b, ", subskiplsn AS \"Skip LSN\"\n");
        if at_least(190000) put(*b, ",  pg_catalog.obj_description(oid, 'pg_subscription') AS \"Description\"\n");
    }

    put(*b, "FROM pg_catalog.pg_subscription\nWHERE subdbid = (SELECT oid\n                 FROM pg_catalog.pg_database\n                 WHERE datname = pg_catalog.current_database())");

    if !validate_pattern(*b, pattern, true, false, "", "subname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, "List of subscriptions", psql.popt);

    return true;
}

// \da \dA \db \df \dT \do

// Runs the query and prints its rows under `title`.
run_and_print :: (b: *String_Builder, title: string) -> bool {
    r := run_one(b);
    if !r return false;

    defer free_query_result(r);

    print_result(r, title, psql.popt);

    return true;
}

describe_aggregates :: (pattern: Pattern, _verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  p.proname AS \"Name\",\n  pg_catalog.format_type(p.prorettype, NULL) AS \"Result data type\",\n  CASE WHEN p.pronargs = 0\n    THEN CAST('*' AS pg_catalog.text)\n    ELSE pg_catalog.pg_get_function_arguments(p.oid)\n  END AS \"Argument data types\",\n  pg_catalog.obj_description(p.oid, 'pg_proc') as \"Description\"\nFROM pg_catalog.pg_proc p\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace\nWHERE p.prokind = 'a'\n");

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "p.proname", "", "pg_catalog.pg_function_is_visible(p.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2, 4;");

    return run_and_print(*b, "List of aggregate functions");
}

describe_access_methods :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT amname AS \"Name\",\n  CASE amtype WHEN 'i' THEN 'Index' WHEN 't' THEN 'Table' END AS \"Type\"");

    if verbose put(*b, ",\n  amhandler AS \"Handler\",\n  pg_catalog.obj_description(oid, 'pg_am') AS \"Description\"");

    put(*b, "\nFROM pg_catalog.pg_am\n");

    if !validate_pattern(*b, pattern, false, false, "", "amname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of access methods");
}

describe_tablespaces :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT spcname AS \"Name\",\n  pg_catalog.pg_get_userbyid(spcowner) AS \"Owner\",\n  pg_catalog.pg_tablespace_location(oid) AS \"Location\"");

    if verbose {
        put(*b, ",\n  ");
        acl_column(*b, "spcacl");
        put(*b, ",\n  spcoptions AS \"Options\",\n  pg_catalog.pg_size_pretty(pg_catalog.pg_tablespace_size(oid)) AS \"Size\",\n  pg_catalog.shobj_description(oid, 'pg_tablespace') AS \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_tablespace\n");

    if !validate_pattern(*b, pattern, false, false, "", "spcname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of tablespaces");
}

// Variant type names the grammar accepts, mapped to the names pg_type and format_type() use.
map_typename_pattern :: (pattern: Pattern) -> Pattern {
    if !pattern.set return pattern;

    pairs := string.[
        "decimal", "numeric",
        "float", "double precision",
        "int", "integer",
        "bool[]", "boolean[]",
        "decimal[]", "numeric[]",
        "float[]", "double precision[]",
        "float4[]", "real[]",
        "float8[]", "double precision[]",
        "int[]", "integer[]",
        "int2[]", "smallint[]",
        "int4[]", "integer[]",
        "int8[]", "bigint[]",
        "time[]", "time without time zone[]",
        "timetz[]", "time with time zone[]",
        "timestamp[]", "timestamp without time zone[]",
        "timestamptz[]", "timestamp with time zone[]",
        "varbit[]", "bit varying[]",
        "varchar[]", "character varying[]"
    ];

    for 0..pairs.count / 2 - 1 {
        if equal_nocase(pattern.text, pairs[it * 2]) return Pattern.{ pairs[it * 2 + 1], true };
    }

    return pattern;
}

// The WHERE clauses for the argument-type patterns of \df and \do.
argument_type_patterns :: (b: *String_Builder, args: [] string) -> bool {
    for args {
        if it != "-" {
            if !validate_pattern(b, map_typename_pattern(Pattern.{ it, true }), true, false,
                tprint("nt%.nspname", it_index), tprint("t%.typname", it_index),
                tprint("pg_catalog.format_type(t%.oid, NULL)", it_index),
                tprint("pg_catalog.pg_type_is_visible(t%.oid)", it_index), 3) return false;
        } else {
            putf(b, "  AND t%.typname IS NULL\n", it_index);
        }
    }

    return true;
}

describe_functions :: (functypes: string, pattern: Pattern, args: [] string, verbose: bool, show_system: bool) -> bool {
    for 0..functypes.count - 1 {
        if !contains("anptwS+", slice(functypes, it, 1)) {
            print_error("\\df only takes [anptwS+] as options");
            return true;
        }
    }

    show_aggregate := contains(functypes, "a");
    show_normal := contains(functypes, "n");
    show_procedure := contains(functypes, "p");
    show_trigger := contains(functypes, "t");
    show_window := contains(functypes, "w");

    if !show_aggregate && !show_normal && !show_procedure && !show_trigger && !show_window {
        show_aggregate = true;
        show_normal = true;
        show_trigger = true;
        show_window = true;
        show_procedure = true;
    }

    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  p.proname as \"Name\",\n  pg_catalog.pg_get_function_result(p.oid) as \"Result data type\",\n  pg_catalog.pg_get_function_arguments(p.oid) as \"Argument data types\",\n CASE p.prokind\n  WHEN 'a' THEN 'agg'\n  WHEN 'w' THEN 'window'\n  WHEN 'p' THEN 'proc'\n  ELSE 'func'\n END as \"Type\"");

    if verbose {
        put(*b, ",\n CASE\n  WHEN p.provolatile = 'i' THEN 'immutable'\n  WHEN p.provolatile = 's' THEN 'stable'\n  WHEN p.provolatile = 'v' THEN 'volatile'\n END as \"Volatility\"");
        put(*b, ",\n CASE\n  WHEN p.proparallel = 'r' THEN 'restricted'\n  WHEN p.proparallel = 's' THEN 'safe'\n  WHEN p.proparallel = 'u' THEN 'unsafe'\n END as \"Parallel\"");
        put(*b, ",\n pg_catalog.pg_get_userbyid(p.proowner) as \"Owner\",\n CASE WHEN prosecdef THEN 'definer' ELSE 'invoker' END AS \"Security\"");
        if at_least(180000) put(*b, ",\n CASE WHEN p.proleakproof THEN 'yes' ELSE 'no' END AS \"Leakproof?\"");

        put(*b, ",\n ");
        acl_column(*b, "p.proacl");
        put(*b, ",\n l.lanname as \"Language\"");
        put(*b, ",\n CASE WHEN l.lanname IN ('internal', 'c') THEN p.prosrc END as \"Internal name\"");
        put(*b, ",\n pg_catalog.obj_description(p.oid, 'pg_proc') as \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_proc p\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace\n");

    for 0..args.count - 1 {
        putf(*b, "     LEFT JOIN pg_catalog.pg_type t% ON t%.oid = p.proargtypes[%]\n     LEFT JOIN pg_catalog.pg_namespace nt% ON nt%.oid = t%.typnamespace\n", it, it, it, it, it, it);
    }

    if verbose put(*b, "     LEFT JOIN pg_catalog.pg_language l ON l.oid = p.prolang\n");

    have_where := false;

    clause :: (b: *String_Builder, have_where: *bool, text: string) {
        if have_where.* put(b, "      AND ");
        else {
            put(b, "WHERE ");
            have_where.* = true;
        }

        put(b, text);
    }

    if show_normal && show_aggregate && show_procedure && show_trigger && show_window {} else if show_normal {
        if !show_aggregate clause(*b, *have_where, "p.prokind <> 'a'\n");
        if !show_procedure clause(*b, *have_where, "p.prokind <> 'p'\n");
        if !show_trigger clause(*b, *have_where, "p.prorettype <> 'pg_catalog.trigger'::pg_catalog.regtype\n");
        if !show_window clause(*b, *have_where, "p.prokind <> 'w'\n");
    } else {
        needs_or := false;

        put(*b, "WHERE (\n       ");
        have_where = true;

        if show_aggregate {
            put(*b, "p.prokind = 'a'\n");
            needs_or = true;
        }

        if show_trigger {
            if needs_or put(*b, "       OR ");
            put(*b, "p.prorettype = 'pg_catalog.trigger'::pg_catalog.regtype\n");
            needs_or = true;
        }

        if show_procedure {
            if needs_or put(*b, "       OR ");
            put(*b, "p.prokind = 'p'\n");
            needs_or = true;
        }

        if show_window {
            if needs_or put(*b, "       OR ");
            put(*b, "p.prokind = 'w'\n");
        }

        put(*b, "      )\n");
    }

    if !validate_pattern(*b, pattern, have_where, false, "n.nspname", "p.proname", "", "pg_catalog.pg_function_is_visible(p.oid)", 3) return false;
    if !argument_type_patterns(*b, args) return false;

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    put(*b, "ORDER BY 1, 2, 4;");

    return run_and_print(*b, "List of functions");
}

describe_types :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  pg_catalog.format_type(t.oid, NULL) AS \"Name\",\n");

    if verbose {
        put(*b, "  t.typname AS \"Internal name\",\n  CASE WHEN t.typrelid != 0\n      THEN CAST('tuple' AS pg_catalog.text)\n    WHEN t.typlen < 0\n      THEN CAST('var' AS pg_catalog.text)\n    ELSE CAST(t.typlen AS pg_catalog.text)\n  END AS \"Size\",\n  pg_catalog.array_to_string(\n      ARRAY(\n          SELECT e.enumlabel\n          FROM pg_catalog.pg_enum e\n          WHERE e.enumtypid = t.oid\n          ORDER BY e.enumsortorder\n      ),\n      E'\\n'\n  ) AS \"Elements\",\n  pg_catalog.pg_get_userbyid(t.typowner) AS \"Owner\",\n");
        acl_column(*b, "t.typacl");
        put(*b, ",\n  ");
    }

    put(*b, "  pg_catalog.obj_description(t.oid, 'pg_type') as \"Description\"\n");
    put(*b, "FROM pg_catalog.pg_type t\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace\n");
    put(*b, "WHERE (t.typrelid = 0 OR (SELECT c.relkind = 'c' FROM pg_catalog.pg_class c WHERE c.oid = t.typrelid))\n");

    if !pattern.set || !contains(pattern.text, "[]") {
        put(*b, "  AND NOT EXISTS(SELECT 1 FROM pg_catalog.pg_type el WHERE el.oid = t.typelem AND el.typarray = t.oid)\n");
    }

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, map_typename_pattern(pattern), true, false, "n.nspname", "t.typname", "pg_catalog.format_type(t.oid, NULL)", "pg_catalog.pg_type_is_visible(t.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of data types");
}

describe_operators :: (pattern: Pattern, args_in: [] string, verbose: bool, show_system: bool) -> bool {
    args := args_in;
    if args.count > 2 args.count = 2;

    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n  o.oprname AS \"Name\",\n  CASE WHEN o.oprkind='l' THEN NULL ELSE pg_catalog.format_type(o.oprleft, NULL) END AS \"Left arg type\",\n  CASE WHEN o.oprkind='r' THEN NULL ELSE pg_catalog.format_type(o.oprright, NULL) END AS \"Right arg type\",\n  pg_catalog.format_type(o.oprresult, NULL) AS \"Result type\",\n");

    if verbose put(*b, "  o.oprcode AS \"Function\",\n");

    put(*b, "  coalesce(pg_catalog.obj_description(o.oid, 'pg_operator'),\n           pg_catalog.obj_description(o.oprcode, 'pg_proc')) AS \"Description\"\nFROM pg_catalog.pg_operator o\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = o.oprnamespace\n");

    if args.count >= 2 {
        put(*b, "     LEFT JOIN pg_catalog.pg_type t0 ON t0.oid = o.oprleft\n     LEFT JOIN pg_catalog.pg_namespace nt0 ON nt0.oid = t0.typnamespace\n     LEFT JOIN pg_catalog.pg_type t1 ON t1.oid = o.oprright\n     LEFT JOIN pg_catalog.pg_namespace nt1 ON nt1.oid = t1.typnamespace\n");
    } else if args.count == 1 {
        put(*b, "     LEFT JOIN pg_catalog.pg_type t0 ON t0.oid = o.oprright\n     LEFT JOIN pg_catalog.pg_namespace nt0 ON nt0.oid = t0.typnamespace\n");
    }

    if !show_system && !pattern.set put(*b, "WHERE n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, !show_system && !pattern.set, true, "n.nspname", "o.oprname", "", "pg_catalog.pg_operator_is_visible(o.oid)", 3) return false;

    if args.count == 1 put(*b, "  AND o.oprleft = 0\n");

    if !argument_type_patterns(*b, args) return false;

    put(*b, "ORDER BY 1, 2, 3, 4;");

    return run_and_print(*b, "List of operators");
}

// \du \drds \drg \dL \dD \dc \dconfig \dy \dX \dC \dO \dn

describe_roles :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT r.rolname, r.rolsuper, r.rolinherit,\n  r.rolcreaterole, r.rolcreatedb, r.rolcanlogin,\n  r.rolconnlimit, r.rolvaliduntil");
    // Up to PostgreSQL 15 the list shows each role's memberships; from 16 \drg does.
    member_column := !at_least(160000);
    if member_column put(*b, ",\n  ARRAY(SELECT b.rolname\n        FROM pg_catalog.pg_auth_members m\n        JOIN pg_catalog.pg_roles b ON (m.roleid = b.oid)\n        WHERE m.member = r.oid) as memberof");
    if verbose put(*b, "\n, pg_catalog.shobj_description(r.oid, 'pg_authid') AS description");

    put(*b, "\n, r.rolreplication\n, r.rolbypassrls\nFROM pg_catalog.pg_roles r\n");

    if !show_system && !pattern.set put(*b, "WHERE r.rolname !~ '^pg_'\n");

    if !validate_pattern(*b, pattern, false, false, "", "r.rolname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    cont: Content;

    content_init(*cont, "List of roles");
    add_header(*cont, "Role name");
    add_header(*cont, "Attributes");
    if member_column add_header(*cont, "Member of");
    if verbose add_header(*cont, "Description");

    extra := (ifx member_column then 1 else 0) + (ifx verbose then 1 else 0);

    for i: 0..r.row_count - 1 {
        add_cell(*cont, cell(r, i, 0));

        attrs: [..] string;
        attrs.allocator = temp;

        if is_true(r, i, 1) array_add(*attrs, "Superuser");
        if !is_true(r, i, 2) array_add(*attrs, "No inheritance");
        if is_true(r, i, 3) array_add(*attrs, "Create role");
        if is_true(r, i, 4) array_add(*attrs, "Create DB");
        if !is_true(r, i, 5) array_add(*attrs, "Cannot login");
        if is_true(r, i, 8 + extra) array_add(*attrs, "Replication");
        if is_true(r, i, 9 + extra) array_add(*attrs, "Bypass RLS");

        text := join(..attrs, separator = ", ",, temp);

        conns := to_int(cell(r, i, 6));

        if conns >= 0 {
            if text.count text = tprint("%\n", text);

            if conns == 0 text = join(text, "No connections");
            else if conns == 1 text = join(text, "1 connection");
            else text = join(text, tprint("% connections", conns));
        }

        if cell(r, i, 7).count {
            if text.count text = tprint("%\n", text);
            text = tprint("%Password valid until %", text, cell(r, i, 7));
        }

        add_cell(*cont, text);
        if member_column add_cell(*cont, cell(r, i, 8));
        if verbose add_cell(*cont, cell(r, i, ifx member_column then 9 else 8));
    }

    emit_content(*cont, psql.popt);

    return true;
}

list_db_role_settings :: (pattern: Pattern, pattern2: Pattern) -> bool {
    b := new_sql();

    put(*b, "SELECT rolname AS \"Role\", datname AS \"Database\",\npg_catalog.array_to_string(setconfig, E'\\n') AS \"Settings\"\nFROM pg_catalog.pg_db_role_setting s\nLEFT JOIN pg_catalog.pg_database d ON d.oid = setdatabase\nLEFT JOIN pg_catalog.pg_roles r ON r.oid = setrole\n");

    ok, have_where := validate_pattern(*b, pattern, false, false, "", "r.rolname", "", "", 1);
    if !ok return false;

    if !validate_pattern(*b, pattern2, have_where, false, "", "d.datname", "", "", 1) return false;

    put(*b, "ORDER BY 1, 2;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 && !psql.quiet {
        if pattern.set && pattern2.set print_error(tprint("Did not find any settings for role \"%\" and database \"%\".", pattern.text, pattern2.text));
        else if pattern.set print_error(tprint("Did not find any settings for role \"%\".", pattern.text));
        else print_error("Did not find any settings.");
    } else {
        print_result(r, "List of settings", psql.popt);
    }

    return true;
}

describe_role_grants :: (pattern: Pattern, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT m.rolname AS \"Role name\", r.rolname AS \"Member of\",\n  pg_catalog.concat_ws(', ',\n    CASE WHEN pam.admin_option THEN 'ADMIN' END,\n    CASE WHEN pam.inherit_option THEN 'INHERIT' END,\n    CASE WHEN pam.set_option THEN 'SET' END\n  ) AS \"Options\",\n  g.rolname AS \"Grantor\"\nFROM pg_catalog.pg_roles m\n     JOIN pg_catalog.pg_auth_members pam ON (pam.member = m.oid)\n     LEFT JOIN pg_catalog.pg_roles r ON (pam.roleid = r.oid)\n     LEFT JOIN pg_catalog.pg_roles g ON (pam.grantor = g.oid)\n");

    if !show_system && !pattern.set put(*b, "WHERE m.rolname !~ '^pg_'\n");

    if !validate_pattern(*b, pattern, false, false, "", "m.rolname", "", "", 1) return false;

    put(*b, "ORDER BY 1, 2, 4;\n");

    return run_and_print(*b, "List of role grants");
}

list_languages :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT l.lanname AS \"Name\",\n       pg_catalog.pg_get_userbyid(l.lanowner) as \"Owner\",\n       l.lanpltrusted AS \"Trusted\"");

    if verbose {
        put(*b, ",\n       NOT l.lanispl AS \"Internal language\",\n       l.lanplcallfoid::pg_catalog.regprocedure AS \"Call handler\",\n       l.lanvalidator::pg_catalog.regprocedure AS \"Validator\",\n       l.laninline::pg_catalog.regprocedure AS \"Inline handler\",\n       ");
        acl_column(*b, "l.lanacl");
    }

    put(*b, ",\n       d.description AS \"Description\"\nFROM pg_catalog.pg_language l\nLEFT JOIN pg_catalog.pg_description d\n  ON d.classoid = l.tableoid AND d.objoid = l.oid\n  AND d.objsubid = 0\n");

    if pattern.set {
        if !validate_pattern(*b, pattern, false, false, "", "l.lanname", "", "", 2) return false;
    }

    if !show_system && !pattern.set put(*b, "WHERE l.lanplcallfoid != 0\n");

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of languages");
}

list_domains :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname as \"Schema\",\n       t.typname as \"Name\",\n       pg_catalog.format_type(t.typbasetype, t.typtypmod) as \"Type\",\n       (SELECT c.collname FROM pg_catalog.pg_collation c, pg_catalog.pg_type bt\n        WHERE c.oid = t.typcollation AND bt.oid = t.typbasetype AND t.typcollation <> bt.typcollation) as \"Collation\",\n       CASE WHEN t.typnotnull THEN 'not null' END as \"Nullable\",\n       t.typdefault as \"Default\",\n       pg_catalog.array_to_string(ARRAY(\n         SELECT pg_catalog.pg_get_constraintdef(r.oid, true) FROM pg_catalog.pg_constraint r WHERE t.oid = r.contypid AND r.contype = 'c' ORDER BY r.conname\n       ), ' ') as \"Check\"");

    if verbose {
        put(*b, ",\n  ");
        acl_column(*b, "t.typacl");
        put(*b, ",\n       d.description as \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_type t\n     LEFT JOIN pg_catalog.pg_namespace n ON n.oid = t.typnamespace\n");
    if verbose put(*b, "     LEFT JOIN pg_catalog.pg_description d ON d.classoid = t.tableoid AND d.objoid = t.oid AND d.objsubid = 0\n");

    put(*b, "WHERE t.typtype = 'd'\n");

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "t.typname", "", "pg_catalog.pg_type_is_visible(t.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of domains");
}

list_conversions :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname AS \"Schema\",\n       c.conname AS \"Name\",\n       pg_catalog.pg_encoding_to_char(c.conforencoding) AS \"Source\",\n       pg_catalog.pg_encoding_to_char(c.contoencoding) AS \"Destination\",\n       CASE WHEN c.condefault THEN 'yes'\n       ELSE 'no' END AS \"Default?\"");

    if verbose put(*b, ",\n       d.description AS \"Description\"");

    put(*b, "\nFROM pg_catalog.pg_conversion c\n     JOIN pg_catalog.pg_namespace n ON n.oid = c.connamespace\n");
    if verbose put(*b, "LEFT JOIN pg_catalog.pg_description d ON d.classoid = c.tableoid\n          AND d.objoid = c.oid AND d.objsubid = 0\n");

    put(*b, "WHERE true\n");

    if !show_system && !pattern.set put(*b, "  AND n.nspname <> 'pg_catalog'\n  AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.conname", "", "pg_catalog.pg_conversion_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of conversions");
}

describe_configuration_parameters :: (pattern: Pattern, verbose: bool, _show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT s.name AS \"Parameter\", pg_catalog.current_setting(s.name) AS \"Value\"");

    if verbose {
        put(*b, ", s.vartype AS \"Type\", s.context AS \"Context\", ");
        acl_column(*b, "p.paracl");
    }

    put(*b, "\nFROM pg_catalog.pg_settings s\n");

    if verbose put(*b, "  LEFT JOIN pg_catalog.pg_parameter_acl p\n  ON pg_catalog.lower(s.name) = p.parname\n");

    if pattern.set process_pattern(*b, pattern, false, false, "", "pg_catalog.lower(s.name)", "", "", 1);
    else put(*b, "WHERE s.source <> 'default' AND\n      s.setting IS DISTINCT FROM s.boot_val\n");

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, ifx pattern.set then "List of configuration parameters" else "List of non-default configuration parameters");
}

list_event_triggers :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT evtname as \"Name\", evtevent as \"Event\", pg_catalog.pg_get_userbyid(e.evtowner) as \"Owner\",\n case evtenabled when 'O' then 'enabled'  when 'R' then 'replica'  when 'A' then 'always'  when 'D' then 'disabled' end as \"Enabled\",\n e.evtfoid::pg_catalog.regproc as \"Function\", pg_catalog.array_to_string(array(select x from pg_catalog.unnest(evttags) as t(x)), ', ') as \"Tags\"");

    if verbose put(*b, ",\npg_catalog.obj_description(e.oid, 'pg_event_trigger') as \"Description\"");

    put(*b, "\nFROM pg_catalog.pg_event_trigger e ");

    if !validate_pattern(*b, pattern, false, false, "", "evtname", "", "", 1) return false;

    put(*b, "ORDER BY 1");

    return run_and_print(*b, "List of event triggers");
}

list_extended_stats :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT \nes.stxnamespace::pg_catalog.regnamespace::pg_catalog.text AS \"Schema\", \nes.stxname AS \"Name\", \npg_catalog.format('%s FROM %s', \n  pg_catalog.pg_get_statisticsobjdef_columns(es.oid), \n  es.stxrelid::pg_catalog.regclass) AS \"Definition\",\nCASE WHEN 'd' = any(es.stxkind) THEN 'defined' \nEND AS \"Ndistinct\", \nCASE WHEN 'f' = any(es.stxkind) THEN 'defined' \nEND AS \"Dependencies\",\nCASE WHEN 'm' = any(es.stxkind) THEN 'defined' \nEND AS \"MCV\" ");

    if verbose && at_least(190000) put(*b, ", \npg_catalog.obj_description(oid, 'pg_statistic_ext') AS \"Description\"\n");

    put(*b, " \nFROM pg_catalog.pg_statistic_ext es \n");

    if !validate_pattern(*b, pattern, false, false, "es.stxnamespace::pg_catalog.regnamespace::pg_catalog.text", "es.stxname", "", "pg_catalog.pg_statistics_obj_is_visible(es.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of extended statistics");
}

list_casts :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT pg_catalog.format_type(castsource, NULL) AS \"Source type\",\n       pg_catalog.format_type(casttarget, NULL) AS \"Target type\",\n       CASE WHEN c.castmethod = 'b' THEN '(binary coercible)'\n            WHEN c.castmethod = 'i' THEN '(with inout)'\n            ELSE p.proname\n       END AS \"Function\",\n       CASE WHEN c.castcontext = 'e' THEN 'no'\n            WHEN c.castcontext = 'a' THEN 'in assignment'\n            ELSE 'yes'\n       END AS \"Implicit?\"");

    if verbose put(*b, ",\n       d.description AS \"Description\"");

    put(*b, "\nFROM pg_catalog.pg_cast c LEFT JOIN pg_catalog.pg_proc p\n     ON c.castfunc = p.oid\n     LEFT JOIN pg_catalog.pg_type ts\n     ON c.castsource = ts.oid\n     LEFT JOIN pg_catalog.pg_namespace ns\n     ON ns.oid = ts.typnamespace\n     LEFT JOIN pg_catalog.pg_type tt\n     ON c.casttarget = tt.oid\n     LEFT JOIN pg_catalog.pg_namespace nt\n     ON nt.oid = tt.typnamespace\n");
    if verbose put(*b, "     LEFT JOIN pg_catalog.pg_description d\n     ON d.classoid = c.tableoid AND d.objoid = c.oid AND d.objsubid = 0\n");

    put(*b, "WHERE ( (true");
    if !validate_pattern(*b, pattern, true, false, "ns.nspname", "ts.typname", "pg_catalog.format_type(ts.oid, NULL)", "pg_catalog.pg_type_is_visible(ts.oid)", 3) return false;

    put(*b, ") OR (true");
    if !validate_pattern(*b, pattern, true, false, "nt.nspname", "tt.typname", "pg_catalog.format_type(tt.oid, NULL)", "pg_catalog.pg_type_is_visible(tt.oid)", 3) return false;

    put(*b, ") )\nORDER BY 1, 2;");

    return run_and_print(*b, "List of casts");
}

list_collations :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    if !at_least(160000) {
        // PostgreSQL 15 lists the locales before the provider and has no ICU rules.
        put(*b, "SELECT\n  n.nspname AS \"Schema\",\n  c.collname AS \"Name\",\n  c.collcollate AS \"Collate\",\n  c.collctype AS \"Ctype\",\n  c.colliculocale AS \"ICU Locale\",\n  CASE c.collprovider WHEN 'd' THEN 'default' WHEN 'c' THEN 'libc' WHEN 'i' THEN 'icu' END AS \"Provider\",\n  CASE WHEN c.collisdeterministic THEN 'yes' ELSE 'no' END AS \"Deterministic?\"");
    } else {
        put(*b, "SELECT\n  n.nspname AS \"Schema\",\n  c.collname AS \"Name\",\n  CASE c.collprovider WHEN 'd' THEN 'default' WHEN 'b' THEN 'builtin' WHEN 'c' THEN 'libc' WHEN 'i' THEN 'icu' END AS \"Provider\",\n  c.collcollate AS \"Collate\",\n  c.collctype AS \"Ctype\",\n");

        if at_least(170000) put(*b, "  c.colllocale AS \"Locale\",\n");
        else put(*b, "  c.colliculocale AS \"ICU Locale\",\n");

        put(*b, "  c.collicurules AS \"ICU Rules\",\n");
        put(*b, "  CASE WHEN c.collisdeterministic THEN 'yes' ELSE 'no' END AS \"Deterministic?\"");
    }

    if verbose put(*b, ",\n  pg_catalog.obj_description(c.oid, 'pg_collation') AS \"Description\"");

    put(*b, "\nFROM pg_catalog.pg_collation c, pg_catalog.pg_namespace n\nWHERE n.oid = c.collnamespace\n");

    if !show_system && !pattern.set put(*b, "      AND n.nspname <> 'pg_catalog'\n      AND n.nspname <> 'information_schema'\n");

    put(*b, "      AND c.collencoding IN (-1, pg_catalog.pg_char_to_encoding(pg_catalog.getdatabaseencoding()))\n");

    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.collname", "", "pg_catalog.pg_collation_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of collations");
}

list_schemas :: (pattern: Pattern, verbose: bool, show_system: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname AS \"Name\",\n  pg_catalog.pg_get_userbyid(n.nspowner) AS \"Owner\"");

    if verbose {
        put(*b, ",\n  ");
        acl_column(*b, "n.nspacl");
        put(*b, ",\n  pg_catalog.obj_description(n.oid, 'pg_namespace') AS \"Description\"");
    }

    put(*b, "\nFROM pg_catalog.pg_namespace n\n");

    if !show_system && !pattern.set put(*b, "WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'\n");

    if !validate_pattern(*b, pattern, !show_system && !pattern.set, false, "", "n.nspname", "", "", 2) return false;

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    footers: [..] string;
    footers.allocator = temp;

    if pattern.set {
        q := psql_exec(tprint("SELECT pubname \nFROM pg_catalog.pg_publication p\n     JOIN pg_catalog.pg_publication_namespace pn ON p.oid = pn.pnpubid\n     JOIN pg_catalog.pg_namespace n ON n.oid = pn.pnnspid \nWHERE n.nspname = '%'\nORDER BY 1", pattern.text));
        if !q return false;

        defer free_query_result(q);

        if q.row_count > 0 {
            array_add(*footers, ifx at_least(190000) then "Included in publications:" else "Publications:");
            for i: 0..q.row_count - 1 array_add(*footers, tprint("    \"%\"", cell(q, i, 0)));
        }
    }

    print_result(r, "List of schemas", psql.popt, footers);

    return true;
}

// \dF* \de* \dx \dA*

list_ts_parsers :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    if verbose put(*b, "SELECT p.oid,\n  n.nspname,\n  p.prsname\nFROM pg_catalog.pg_ts_parser p\nLEFT JOIN pg_catalog.pg_namespace n ON n.oid = p.prsnamespace\n");
    else put(*b, "SELECT\n  n.nspname as \"Schema\",\n  p.prsname as \"Name\",\n  pg_catalog.obj_description(p.oid, 'pg_ts_parser') as \"Description\"\nFROM pg_catalog.pg_ts_parser p\nLEFT JOIN pg_catalog.pg_namespace n ON n.oid = p.prsnamespace\n");

    if !validate_pattern(*b, pattern, false, false, "n.nspname", "p.prsname", "", "pg_catalog.pg_ts_parser_is_visible(p.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    if !verbose return run_and_print(*b, "List of text search parsers");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 {
        if !psql.quiet {
            if pattern.set print_error(tprint("Did not find any text search parser named \"%\".", pattern.text));
            else print_error("Did not find any text search parsers.");
        }

        return false;
    }

    for i: 0..r.row_count - 1 {
        if !describe_one_ts_parser(cell(r, i, 0), ifx is_null(r, i, 1) then "" else cell(r, i, 1), !is_null(r, i, 1), cell(r, i, 2)) return false;
    }

    return true;
}

describe_one_ts_parser :: (oid: string, nspname: string, has_schema: bool, prsname: string) -> bool {
    b := new_sql();
    functions := string.["Start parse", "prsstart", "Get next token", "prstoken", "End parse", "prsend", "Get headline", "prsheadline", "Get token types", "prslextype"];

    for 0..4 {
        label := functions[it * 2];
        column := functions[it * 2 + 1];

        if it == 0 {
            putf(*b, "SELECT '%' AS \"Method\",\n   p.prsstart::pg_catalog.regproc AS \"Function\",\n   pg_catalog.obj_description(p.prsstart, 'pg_proc') as \"Description\"\n FROM pg_catalog.pg_ts_parser p\n WHERE p.oid = '%'\n", label, oid);
        } else {
            putf(*b, "UNION ALL\nSELECT '%',\n   p.%::pg_catalog.regproc,\n   pg_catalog.obj_description(p.%, 'pg_proc')\n FROM pg_catalog.pg_ts_parser p\n WHERE p.oid = '%'\n", label, column, column, oid);
        }
    }

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    qualified := ifx has_schema then tprint("%.%", nspname, prsname) else prsname;

    print_result(r, tprint("Text search parser \"%\"", qualified), psql.popt, .[], false);

    q := psql_exec(tprint("SELECT t.alias as \"Token name\",\n  t.description as \"Description\"\nFROM pg_catalog.ts_token_type( '%'::pg_catalog.oid ) as t\nORDER BY 1;", oid));
    if !q return false;

    defer free_query_result(q);

    print_result(q, tprint("Token types for parser \"%\"", qualified), psql.popt);

    return true;
}

list_ts_dictionaries :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  n.nspname as \"Schema\",\n  d.dictname as \"Name\",\n");

    if verbose {
        put(*b, "  ( SELECT COALESCE(nt.nspname, '(null)')::pg_catalog.text || '.' || t.tmplname FROM\n    pg_catalog.pg_ts_template t\n    LEFT JOIN pg_catalog.pg_namespace nt ON nt.oid = t.tmplnamespace\n    WHERE d.dicttemplate = t.oid ) AS  \"Template\",\n  d.dictinitoption as \"Init options\",\n");
    }

    put(*b, "  pg_catalog.obj_description(d.oid, 'pg_ts_dict') as \"Description\"\n");
    put(*b, "FROM pg_catalog.pg_ts_dict d\nLEFT JOIN pg_catalog.pg_namespace n ON n.oid = d.dictnamespace\n");

    if !validate_pattern(*b, pattern, false, false, "n.nspname", "d.dictname", "", "pg_catalog.pg_ts_dict_is_visible(d.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of text search dictionaries");
}

list_ts_templates :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    if verbose put(*b, "SELECT\n  n.nspname AS \"Schema\",\n  t.tmplname AS \"Name\",\n  t.tmplinit::pg_catalog.regproc AS \"Init\",\n  t.tmpllexize::pg_catalog.regproc AS \"Lexize\",\n  pg_catalog.obj_description(t.oid, 'pg_ts_template') AS \"Description\"\n");
    else put(*b, "SELECT\n  n.nspname AS \"Schema\",\n  t.tmplname AS \"Name\",\n  pg_catalog.obj_description(t.oid, 'pg_ts_template') AS \"Description\"\n");

    put(*b, "FROM pg_catalog.pg_ts_template t\nLEFT JOIN pg_catalog.pg_namespace n ON n.oid = t.tmplnamespace\n");

    if !validate_pattern(*b, pattern, false, false, "n.nspname", "t.tmplname", "", "pg_catalog.pg_ts_template_is_visible(t.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of text search templates");
}

list_ts_configs :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    if !verbose {
        put(*b, "SELECT\n   n.nspname as \"Schema\",\n   c.cfgname as \"Name\",\n   pg_catalog.obj_description(c.oid, 'pg_ts_config') as \"Description\"\nFROM pg_catalog.pg_ts_config c\nLEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.cfgnamespace\n");
        if !validate_pattern(*b, pattern, false, false, "n.nspname", "c.cfgname", "", "pg_catalog.pg_ts_config_is_visible(c.oid)", 3) return false;
        put(*b, "ORDER BY 1, 2;");
        return run_and_print(*b, "List of text search configurations");
    }

    put(*b, "SELECT c.oid, c.cfgname,\n   n.nspname,\n   p.prsname,\n   np.nspname as pnspname\nFROM pg_catalog.pg_ts_config c\n   LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.cfgnamespace,\n pg_catalog.pg_ts_parser p\n   LEFT JOIN pg_catalog.pg_namespace np ON np.oid = p.prsnamespace\nWHERE  p.oid = c.cfgparser\n");
    if !validate_pattern(*b, pattern, true, false, "n.nspname", "c.cfgname", "", "pg_catalog.pg_ts_config_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 3, 2;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 {
        if !psql.quiet {
            if pattern.set print_error(tprint("Did not find any text search configuration named \"%\".", pattern.text));
            else print_error("Did not find any text search configurations.");
        }

        return false;
    }

    for i: 0..r.row_count - 1 {
        oid := cell(r, i, 0);
        cfgname := cell(r, i, 1);
        prsname := cell(r, i, 3);

        q := psql_exec(tprint("SELECT\n  ( SELECT t.alias FROM\n    pg_catalog.ts_token_type(c.cfgparser) AS t\n    WHERE t.tokid = m.maptokentype ) AS \"Token\",\n  pg_catalog.btrim(\n    ARRAY( SELECT mm.mapdict::pg_catalog.regdictionary\n           FROM pg_catalog.pg_ts_config_map AS mm\n           WHERE mm.mapcfg = m.mapcfg AND mm.maptokentype = m.maptokentype\n           ORDER BY mapcfg, maptokentype, mapseqno\n    ) :: pg_catalog.text,\n  '{}') AS \"Dictionaries\"\nFROM pg_catalog.pg_ts_config AS c, pg_catalog.pg_ts_config_map AS m\nWHERE c.oid = '%' AND m.mapcfg = c.oid\nGROUP BY m.mapcfg, m.maptokentype, c.cfgparser\nORDER BY 1;", oid));

        if !q return false;

        defer free_query_result(q);

        title := ifx is_null(r, i, 2) then tprint("Text search configuration \"%\"", cfgname) else tprint("Text search configuration \"%.%\"", cell(r, i, 2), cfgname);
        title = tprint("%\n%", title, ifx is_null(r, i, 4) then tprint("Parser: \"%\"", prsname) else tprint("Parser: \"%.%\"", cell(r, i, 4), prsname));

        print_result(q, title, psql.popt, .[], false);
    }

    return true;
}

fdw_options :: (column: string) -> string {
    return tprint("CASE WHEN % IS NULL THEN '' ELSE   '(' || pg_catalog.array_to_string(ARRAY(SELECT   pg_catalog.quote_ident(option_name) ||  ' ' ||   pg_catalog.quote_literal(option_value)  FROM   pg_catalog.pg_options_to_table(%)),  ', ') || ')'   END", column, column);
}

list_foreign_data_wrappers :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT fdw.fdwname AS \"Name\",\n  pg_catalog.pg_get_userbyid(fdw.fdwowner) AS \"Owner\",\n  fdw.fdwhandler::pg_catalog.regproc AS \"Handler\",\n  fdw.fdwvalidator::pg_catalog.regproc AS \"Validator\"");

    if verbose {
        put(*b, ",\n  ");
        acl_column(*b, "fdwacl");
        putf(*b, ",\n % AS \"FDW options\",\n  d.description AS \"Description\" ", fdw_options("fdwoptions"));
    }

    put(*b, "\nFROM pg_catalog.pg_foreign_data_wrapper fdw\n");
    if verbose put(*b, "LEFT JOIN pg_catalog.pg_description d\n       ON d.classoid = fdw.tableoid AND d.objoid = fdw.oid AND d.objsubid = 0\n");

    if !validate_pattern(*b, pattern, false, false, "", "fdwname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of foreign-data wrappers");
}

list_foreign_servers :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT s.srvname AS \"Name\",\n  pg_catalog.pg_get_userbyid(s.srvowner) AS \"Owner\",\n  f.fdwname AS \"Foreign-data wrapper\"");

    if verbose {
        put(*b, ",\n  ");
        acl_column(*b, "s.srvacl");
        putf(*b, ",\n  s.srvtype AS \"Type\",\n  s.srvversion AS \"Version\",\n  % AS \"FDW options\",\n  d.description AS \"Description\"", fdw_options("srvoptions"));
    }

    put(*b, "\nFROM pg_catalog.pg_foreign_server s\n     JOIN pg_catalog.pg_foreign_data_wrapper f ON f.oid=s.srvfdw\n");
    if verbose put(*b, "LEFT JOIN pg_catalog.pg_description d\n       ON d.classoid = s.tableoid AND d.objoid = s.oid AND d.objsubid = 0\n");

    if !validate_pattern(*b, pattern, false, false, "", "s.srvname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of foreign servers");
}

list_user_mappings :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT um.srvname AS \"Server\",\n  um.usename AS \"User name\"");

    if verbose putf(*b, ",\n % AS \"FDW options\"", fdw_options("umoptions"));

    put(*b, "\nFROM pg_catalog.pg_user_mappings um\n");

    if !validate_pattern(*b, pattern, false, false, "", "um.srvname", "um.usename", "", 1) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of user mappings");
}

list_foreign_tables :: (pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT n.nspname AS \"Schema\",\n  c.relname AS \"Table\",\n  s.srvname AS \"Server\"");

    if verbose putf(*b, ",\n % AS \"FDW options\",\n  d.description AS \"Description\"", fdw_options("ftoptions"));

    put(*b, "\nFROM pg_catalog.pg_foreign_table ft\n  INNER JOIN pg_catalog.pg_class c ON c.oid = ft.ftrelid\n  INNER JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace\n  INNER JOIN pg_catalog.pg_foreign_server s ON s.oid = ft.ftserver\n");
    if verbose put(*b, "   LEFT JOIN pg_catalog.pg_description d\n          ON d.classoid = c.tableoid AND d.objoid = c.oid AND d.objsubid = 0\n");

    if !validate_pattern(*b, pattern, false, false, "n.nspname", "c.relname", "", "pg_catalog.pg_table_is_visible(c.oid)", 3) return false;

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of foreign tables");
}

list_extensions :: (pattern: Pattern) -> bool {
    b := new_sql();

    if at_least(180000) {
        put(*b, "SELECT e.extname AS \"Name\", e.extversion AS \"Version\", ae.default_version AS \"Default version\", n.nspname AS \"Schema\", d.description AS \"Description\"\nFROM pg_catalog.pg_extension e LEFT JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace LEFT JOIN pg_catalog.pg_description d ON d.objoid = e.oid AND d.classoid = 'pg_catalog.pg_extension'::pg_catalog.regclass LEFT JOIN pg_catalog.pg_available_extensions() ae(name, default_version, comment) ON ae.name = e.extname\n");
    } else {
        put(*b, "SELECT e.extname AS \"Name\", e.extversion AS \"Version\", n.nspname AS \"Schema\", c.description AS \"Description\"\nFROM pg_catalog.pg_extension e LEFT JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace LEFT JOIN pg_catalog.pg_description c ON c.objoid = e.oid AND c.classoid = 'pg_catalog.pg_extension'::pg_catalog.regclass\n");
    }

    if !validate_pattern(*b, pattern, false, false, "", "e.extname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    return run_and_print(*b, "List of installed extensions");
}

list_extension_contents :: (pattern: Pattern) -> bool {
    b := new_sql();

    put(*b, "SELECT e.extname, e.oid\nFROM pg_catalog.pg_extension e\n");

    if !validate_pattern(*b, pattern, false, false, "", "e.extname", "", "", 1) return false;

    put(*b, "ORDER BY 1;");

    r := run_one(*b);
    if !r return false;

    defer free_query_result(r);

    if r.row_count == 0 {
        if !psql.quiet {
            if pattern.set print_error(tprint("Did not find any extension named \"%\".", pattern.text));
            else print_error("Did not find any extensions.");
        }

        return false;
    }

    for i: 0..r.row_count - 1 {
        q := psql_exec(tprint("SELECT pg_catalog.pg_describe_object(classid, objid, 0) AS \"Object description\"\nFROM pg_catalog.pg_depend\nWHERE refclassid = 'pg_catalog.pg_extension'::pg_catalog.regclass AND refobjid = '%' AND deptype = 'e'\nORDER BY 1;", cell(r, i, 1)));
        if !q return false;
        defer free_query_result(q);

        print_result(q, tprint("Objects in extension \"%\"", cell(r, i, 0)), psql.popt);
    }

    return true;
}

// The access-method and second patterns shared by the \dA* family.
join_am_pattern :: (b: *String_Builder, am: Pattern) -> ok: bool, have_where: bool {
    have_where := false;

    if am.set {
        ok, added := validate_pattern(b, am, false, false, "", "am.amname", "", "", 1);
        if !ok return false, false;
        have_where = added;
    }

    return true, have_where;
}

list_operator_classes :: (am: Pattern, type_pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  am.amname AS \"AM\",\n  pg_catalog.format_type(c.opcintype, NULL) AS \"Input type\",\n  CASE\n    WHEN c.opckeytype <> 0 AND c.opckeytype <> c.opcintype\n    THEN pg_catalog.format_type(c.opckeytype, NULL)\n    ELSE NULL\n  END AS \"Storage type\",\n  CASE\n    WHEN pg_catalog.pg_opclass_is_visible(c.oid)\n    THEN pg_catalog.format('%I', c.opcname)\n    ELSE pg_catalog.format('%I.%I', n.nspname, c.opcname)\n  END AS \"Operator class\",\n  (CASE WHEN c.opcdefault\n    THEN 'yes'\n    ELSE 'no'\n  END) AS \"Default?\"");

    if verbose put(*b, ",\n  CASE\n    WHEN pg_catalog.pg_opfamily_is_visible(of.oid)\n    THEN pg_catalog.format('%I', of.opfname)\n    ELSE pg_catalog.format('%I.%I', ofn.nspname, of.opfname)\n  END AS \"Operator family\",\n pg_catalog.pg_get_userbyid(c.opcowner) AS \"Owner\"\n");

    put(*b, "\nFROM pg_catalog.pg_opclass c\n  LEFT JOIN pg_catalog.pg_am am on am.oid = c.opcmethod\n  LEFT JOIN pg_catalog.pg_namespace n ON n.oid = c.opcnamespace\n  LEFT JOIN pg_catalog.pg_type t ON t.oid = c.opcintype\n  LEFT JOIN pg_catalog.pg_namespace tn ON tn.oid = t.typnamespace\n");
    if verbose put(*b, "  LEFT JOIN pg_catalog.pg_opfamily of ON of.oid = c.opcfamily\n  LEFT JOIN pg_catalog.pg_namespace ofn ON ofn.oid = of.opfnamespace\n");

    ok, have_where := join_am_pattern(*b, am);
    if !ok return false;

    if type_pattern.set {
        if !validate_pattern(*b, type_pattern, have_where, false, "tn.nspname", "t.typname", "pg_catalog.format_type(t.oid, NULL)", "pg_catalog.pg_type_is_visible(t.oid)", 3) return false;
    }

    put(*b, "ORDER BY 1, 2, 4;");

    return run_and_print(*b, "List of operator classes");
}

list_operator_families :: (am: Pattern, type_pattern: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  am.amname AS \"AM\",\n  CASE\n    WHEN pg_catalog.pg_opfamily_is_visible(f.oid)\n    THEN pg_catalog.format('%I', f.opfname)\n    ELSE pg_catalog.format('%I.%I', n.nspname, f.opfname)\n  END AS \"Operator family\",\n  (SELECT\n     pg_catalog.string_agg(pg_catalog.format_type(oc.opcintype, NULL), ', ')\n   FROM pg_catalog.pg_opclass oc\n   WHERE oc.opcfamily = f.oid) \"Applicable types\"");

    if verbose put(*b, ",\n  pg_catalog.pg_get_userbyid(f.opfowner) AS \"Owner\"\n");

    put(*b, "\nFROM pg_catalog.pg_opfamily f\n  LEFT JOIN pg_catalog.pg_am am on am.oid = f.opfmethod\n  LEFT JOIN pg_catalog.pg_namespace n ON n.oid = f.opfnamespace\n");

    ok, have_where := join_am_pattern(*b, am);
    if !ok return false;

    if type_pattern.set {
        putf(*b, "  % EXISTS (\n    SELECT 1\n    FROM pg_catalog.pg_type t\n    JOIN pg_catalog.pg_opclass oc ON oc.opcintype = t.oid\n    LEFT JOIN pg_catalog.pg_namespace tn ON tn.oid = t.typnamespace\n    WHERE oc.opcfamily = f.oid\n", ifx have_where then "AND" else "WHERE");
        if !validate_pattern(*b, type_pattern, true, false, "tn.nspname", "t.typname", "pg_catalog.format_type(t.oid, NULL)", "pg_catalog.pg_type_is_visible(t.oid)", 3) return false;
        put(*b, "  )\n");
    }

    put(*b, "ORDER BY 1, 2;");

    return run_and_print(*b, "List of operator families");
}

list_opfamily_operators :: (am: Pattern, family: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  am.amname AS \"AM\",\n  CASE\n    WHEN pg_catalog.pg_opfamily_is_visible(of.oid)\n    THEN pg_catalog.format('%I', of.opfname)\n    ELSE pg_catalog.format('%I.%I', nsf.nspname, of.opfname)\n  END AS \"Operator family\",\n  o.amopopr::pg_catalog.regoperator AS \"Operator\"\n,  o.amopstrategy AS \"Strategy\",\n  CASE o.amoppurpose\n    WHEN 'o' THEN 'ordering'\n    WHEN 's' THEN 'search'\n  END AS \"Purpose\"\n");

    if verbose put(*b, ", ofs.opfname AS \"Sort opfamily\"\n");
    if verbose && at_least(180000) put(*b, ", CASE WHEN p.proleakproof THEN 'yes' ELSE 'no' END AS \"Leakproof?\"\n");

    put(*b, "FROM pg_catalog.pg_amop o\n  LEFT JOIN pg_catalog.pg_opfamily of ON of.oid = o.amopfamily\n  LEFT JOIN pg_catalog.pg_am am ON am.oid = of.opfmethod AND am.oid = o.amopmethod\n  LEFT JOIN pg_catalog.pg_namespace nsf ON of.opfnamespace = nsf.oid\n");
    if verbose put(*b, "  LEFT JOIN pg_catalog.pg_opfamily ofs ON ofs.oid = o.amopsortfamily\n");
    if verbose && at_least(180000) put(*b, "  LEFT JOIN pg_catalog.pg_operator op ON op.oid = o.amopopr\n  LEFT JOIN pg_catalog.pg_proc p ON p.oid = op.oprcode\n");

    ok, have_where := join_am_pattern(*b, am);
    if !ok return false;

    if family.set {
        if !validate_pattern(*b, family, have_where, false, "nsf.nspname", "of.opfname", "", "", 3) return false;
    }

    put(*b, "ORDER BY 1, 2,\n  o.amoplefttype = o.amoprighttype DESC,\n  pg_catalog.format_type(o.amoplefttype, NULL),\n  pg_catalog.format_type(o.amoprighttype, NULL),\n  o.amopstrategy;");

    return run_and_print(*b, "List of operators of operator families");
}

list_opfamily_functions :: (am: Pattern, family: Pattern, verbose: bool) -> bool {
    b := new_sql();

    put(*b, "SELECT\n  am.amname AS \"AM\",\n  CASE\n    WHEN pg_catalog.pg_opfamily_is_visible(of.oid)\n    THEN pg_catalog.format('%I', of.opfname)\n    ELSE pg_catalog.format('%I.%I', ns.nspname, of.opfname)\n  END AS \"Operator family\",\n  pg_catalog.format_type(ap.amproclefttype, NULL) AS \"Registered left type\",\n  pg_catalog.format_type(ap.amprocrighttype, NULL) AS \"Registered right type\",\n  ap.amprocnum AS \"Number\"\n");

    if !verbose put(*b, ", p.proname AS \"Function\"\n");
    else put(*b, ", ap.amproc::pg_catalog.regprocedure AS \"Function\"\n");

    put(*b, "FROM pg_catalog.pg_amproc ap\n  LEFT JOIN pg_catalog.pg_opfamily of ON of.oid = ap.amprocfamily\n  LEFT JOIN pg_catalog.pg_am am ON am.oid = of.opfmethod\n  LEFT JOIN pg_catalog.pg_namespace ns ON of.opfnamespace = ns.oid\n  LEFT JOIN pg_catalog.pg_proc p ON ap.amproc = p.oid\n");

    ok, have_where := join_am_pattern(*b, am);
    if !ok return false;

    if family.set {
        if !validate_pattern(*b, family, have_where, false, "ns.nspname", "of.opfname", "", "", 3) return false;
    }

    put(*b, "ORDER BY 1, 2,\n  ap.amproclefttype = ap.amprocrighttype DESC,\n  3, 4, 5;");

    return run_and_print(*b, "List of support functions of operator families");
}
