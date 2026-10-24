// `diff`: compares the structs of a hand-edited `db/schema.jai` with the catalog of the
// migrations' schema database and writes the SQL of a proposed migration. Scaffolding only: it
// never applies anything. docs/postgres/diff.md has the overview.
//
//     proposal, problem := Pg.diff_schema(config, "db/migrations", "db/schema.jai");   // needs a server
//     description, problem := Pg.parse_schema_text(text);                              // just the file
//     proposal, problem := Pg.propose_migration(catalog, description);                 // just the SQL

#scope_export

// One field of a struct in the schema file.
Schema_Field :: struct {
    name: string;
    type_text: string; // as written, `Pg.Nullable(...)` included
    primary_key: bool;
    has_default: bool;
    generated: bool;
    json_text: bool; // `@pg_json`: a `json` column rather than `jsonb`
}

// A struct with an `@pg_table` note.
Schema_Struct :: struct {
    type_name: string;
    schema: string;
    table: string;
    belongs_to: [] string; // the arguments of the `@pg_belongs_to` notes
    fields: [] Schema_Field;
}

// An enum of the schema file: a Postgres enum.
Schema_Enum :: struct {
    type_name: string;
    schema: string;
    name: string;
    labels: [] string; // in declaration order
}

// What `parse_schema_text` reads from the file: the table structs and the enums.
Schema_Description :: struct {
    structs: [] Schema_Struct;
    enums: [] Schema_Enum;
}

// The migration `diff` proposes.
Migration_Proposal :: struct {
    sql: string;
    changes: s64; // statements that change the schema
    todos: s64; // `-- TODO:` comments for what it could not express
}

// Reads the catalog of the migrations' schema database (see schema_databases.jai), compares it
// with the structs in `schema_file`, and returns the proposed SQL; or what went wrong. An empty
// `sql` means nothing differs. Runs on a fiber, like every call that talks to the server.
diff_schema :: (config: Config, migrations_folder: string, schema_file: string) -> Migration_Proposal, problem: string {
    text, read := read_entire_file(schema_file);
    if !read return .{}, tprint("cannot read %", schema_file);

    description, problem := parse_schema_text(text);
    if problem return .{}, tprint("%: %", schema_file, problem);

    catalog, _, catalog_problem := migrated_catalog(config, migrations_folder);
    if catalog_problem return .{}, catalog_problem;

    return propose_migration(catalog, description);
}

// The structs and enums of `db/schema.jai`, which must have the shape `generate` writes: one
// declaration per line of the form `Name :: struct @note(...) {`, then `name: Type; @note` per
// field line, then a `}` in the first column. Structs without `@pg_table` are not tables and are
// skipped. Anything else in a table struct or an enum is a problem naming the line.
parse_schema_text :: (text: string) -> Schema_Description, problem: string {
    structs: [..] Schema_Struct;
    enums: [..] Schema_Enum;

    structs.allocator = temp;
    enums.allocator = temp;

    lines := split(text, "\n");
    doc := "";
    i := 0;

    while i < lines.count {
        line := trim(lines[i]);
        number := i + 1;

        i += 1;

        if begins_with(line, "/// Enum ") && ends_with(line, ".") {
            doc = slice(line, 9, line.count - 10);
            continue;
        }

        if !line || begins_with(line, "//") continue;

        enum_doc := doc;

        doc = "";

        marker := find_index_from_left(line, " :: ");
        if marker < 0 continue;

        name := slice(line, 0, marker);
        rest := trim(advance(line, marker + 4));
        is_struct := keyword_starts(rest, "struct");
        is_enum := keyword_starts(rest, "enum");

        if !is_struct && !is_enum continue;
        if !diff_word(name) continue;

        header := rest;

        if ends_with(header, "{") {
            header = trim(slice(header, 0, header.count - 1));
        } else if ends_with(header, "}") {
            // A declaration on one line: fine unless it is a table.
            if is_struct && contains(header, "@pg_table") return .{}, tprint("line %: put the fields of % on separate lines, as `generate` writes them", number, name);
            continue;
        } else {
            return .{}, tprint("line %: expected `{` at the end of the declaration of %", number, name);
        }

        start := i;

        while i < lines.count && !begins_with(lines[i], "}") i += 1;

        if i >= lines.count return .{}, tprint("line %: % has no closing `}` in the first column", number, name);

        body := array_view(lines, start, i - start);

        i += 1;

        notes := notes_of(header);

        if is_enum {
            declared, problem := parse_enum(name, enum_doc, body, start);
            if problem return .{}, problem;

            array_add(*enums, declared);
            continue;
        }

        schema, table, ok := table_note(notes);
        if !ok continue;

        declared, problem := parse_struct(name, schema, table, notes, body, start);
        if problem return .{}, problem;

        array_add(*structs, declared);
    }

    return .{ structs = structs, enums = enums }, "";
}

// Compares the structs with the catalog and writes the SQL that would bring the migrated schema
// to what the structs say. See docs/postgres/diff.md for what it covers.
propose_migration :: (catalog: Catalog, description: Schema_Description) -> Migration_Proposal, problem: string {
    names, problem := assign_names(catalog);
    if problem return .{}, problem;

    refs := enum_refs(catalog, names, description);

    sections: Proposal_Builder;
    sections.catalog = catalog;
    sections.names = names;
    sections.description = description;
    sections.refs = refs;

    for description.enums propose_enum(*sections, catalog, names, it);

    for description.structs {
        index := relation_index(catalog, it.schema, it.table);

        if index < 0 {
            propose_create(*sections, it);
        } else if !catalog.relations[index].view {
            propose_alter(*sections, catalog.relations[index], it);
        }
    }

    for catalog.relations {
        if it.view continue;

        found := false;
        for s: description.structs if s.schema == it.schema && s.table == it.name found = true;

        if !found {
            line_todo(*sections, tprint("% has no struct in the schema file", qualified(it.schema, it.name)));
            line_comment_statement(*sections, tprint("WARNING: dropping a table destroys its data.\n-- drop table %;", qualified(it.schema, it.name)));
        }
    }

    proposal: Migration_Proposal;
    proposal.changes = sections.changes;
    proposal.todos = sections.todos;

    if sections.changes == 0 && sections.todos == 0 return proposal, "";

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "-- Proposed by the pg tool's `diff` from the structs in db/schema.jai. It is a starting point:\n-- review it, edit it, and resolve every TODO before running `migrate`.\n");

    for string.[builder_to_string(*sections.schemas,, temp), builder_to_string(*sections.enums,, temp), builder_to_string(*sections.creates,, temp), builder_to_string(*sections.alters,, temp), builder_to_string(*sections.notes,, temp)] {
        if it append(*builder, tprint("\n%\n", trim(it)));
    }

    proposal.sql = builder_to_string(*builder);

    return proposal, "";
}

// The Jai type text of a field as the Postgres type that would hold it, or an empty string for a
// type that has no mapping. `refs` resolves the names of enums.
sql_type_of_text :: (text: string, refs: [] Enum_Ref) -> string {
    type := normal_type(text);

    if begins_with(type, "[]") {
        inner := slice(type, 2, type.count - 2);

        if inner == "u8" return "bytea";

        element := sql_type_of_text(inner, refs);
        if !element return "";

        return tprint("%[]", element);
    }

    if type == "bool" return "boolean";
    if type == "s16" return "smallint";
    if type == "s32" return "integer";
    if type == "s64" return "bigint";
    if type == "u32" return "oid";
    if type == "float32" return "real";
    if type == "float64" return "double precision";
    if type == "string" return "text";
    if type == "Uuid" return "uuid";
    if type == "Timestamp" return "timestamptz";
    if type == "Local_Timestamp" return "timestamp";
    if type == "Date" return "date";
    if type == "Time" return "time";
    if type == "Interval" return "interval";
    if type == "Numeric" return "numeric";
    if type == "Json_Raw" || begins_with(type, "Json(") return "jsonb";

    for refs if it.type_name == type return qualified(it.schema, it.name);

    return "";
}

// How an enum of the file is called in Postgres.
Enum_Ref :: struct {
    type_name: string;
    schema: string;
    name: string;
}

#scope_file

Proposal_Builder :: struct {
    catalog: Catalog;
    names: Names;
    description: Schema_Description;
    refs: [] Enum_Ref;

    schemas: String_Builder;
    enums: String_Builder;
    creates: String_Builder;
    alters: String_Builder;
    notes: String_Builder; // what the file cannot express, as comments

    changes: s64;
    todos: s64;
    new_schemas: [..] string;
}

relation_index :: (catalog: Catalog, schema: string, table: string) -> s64 {
    for catalog.relations if it.schema == schema && it.name == table return it_index;

    return -1;
}

keyword_starts :: (text: string, keyword: string) -> bool {
    if !begins_with(text, keyword) return false;
    if text.count == keyword.count return true;

    next := text[keyword.count];

    return next == #char " " || next == #char "{";
}

diff_word :: (name: string) -> bool {
    if !name.count || is_digit(name[0]) return false;

    for name if !(is_alnum(it) && it < 0x80) && it != #char "_" return false;

    return true;
}

// The `@notes` of a declaration header, without the `@`.
notes_of :: (text: string) -> [] string {
    notes: [..] string;
    notes.allocator = temp;

    for token: split(text, " ") {
        if begins_with(token, "@") && token.count > 1 array_add(*notes, advance(token, 1));
    }

    return notes;
}

// The text between the parentheses of `name(...)`, if `note` is that note.
note_argument :: (note: string, name: string) -> string, bool {
    if !begins_with(note, name) || note.count < name.count + 2 return "", false;
    if note[name.count] != #char "(" || !ends_with(note, ")") return "", false;

    return slice(note, name.count + 1, note.count - name.count - 2), true;
}

// `@pg_table(schema.name)`, `@pg_table("name")` or `@pg_table(name)`.
table_note :: (notes: [] string) -> schema: string, table: string, ok: bool {
    for notes {
        argument, found := note_argument(it, "pg_table");
        if !found continue;

        argument = trim(argument);

        if begins_with(argument, "\"") && ends_with(argument, "\"") && argument.count >= 2 argument = slice(argument, 1, argument.count - 2);

        dot := find_index_from_left(argument, ".");

        if dot < 0 return "public", argument, diff_word(argument);

        schema := slice(argument, 0, dot);
        table := advance(argument, dot + 1);

        return schema, table, diff_word(schema) && diff_word(table);
    }

    return "", "", false;
}

parse_enum :: (name: string, doc: string, body: [] string, first_line: s64) -> Schema_Enum, problem: string {
    declared: Schema_Enum;
    declared.type_name = name;
    declared.schema = "public";
    declared.name = snake_case(name);

    dot := find_index_from_left(doc, ".");

    if doc && dot > 0 && diff_word(slice(doc, 0, dot)) && diff_word(advance(doc, dot + 1)) {
        declared.schema = slice(doc, 0, dot);
        declared.name = advance(doc, dot + 1);
    }

    labels: [..] string;
    labels.allocator = temp;

    for body {
        line := trim(it);
        if !line || begins_with(line, "//") continue;

        member := trim(slice(line, 0, line.count - 1));

        if !ends_with(line, ";") || !diff_word(member) {
            return declared, tprint("line %: only plain members (`name;`) are supported in the enum %", first_line + it_index + 1, name);
        }

        array_add(*labels, enum_label(member));
    }

    declared.labels = labels;

    return declared, "";
}

parse_struct :: (name: string, schema: string, table: string, notes: [] string, body: [] string, first_line: s64) -> Schema_Struct, problem: string {
    declared: Schema_Struct;
    declared.type_name = name;
    declared.schema = schema;
    declared.table = table;

    belongs: [..] string;
    belongs.allocator = temp;

    for notes {
        argument, found := note_argument(it, "pg_belongs_to");
        if found array_add(*belongs, argument);
    }

    declared.belongs_to = belongs;

    fields: [..] Schema_Field;
    fields.allocator = temp;

    for body {
        line := trim(it);
        if !line || begins_with(line, "//") continue;

        number := first_line + it_index + 1;
        colon := find_index_from_left(line, ":");
        semicolon := find_index_from_left(line, ";");

        if colon <= 0 || semicolon < colon return declared, tprint("line %: expected `name: Type;` in the struct %", number, name);
        if line[colon + 1] == #char ":" || line[colon + 1] == #char "=" return declared, tprint("line %: the field in % has a constant or a default value, which `diff` does not read", number, name);

        field: Schema_Field;
        field.name = trim(slice(line, 0, colon));
        field.type_text = trim(slice(line, colon + 1, semicolon - colon - 1));

        if contains(field.type_text, "=") return declared, tprint("line %: the field % has a default value, which `diff` does not read", number, field.name);

        if !diff_word(field.name) || !field.type_text {
            return declared, tprint("line %: expected `name: Type;` in the struct %", number, name);
        }

        for note: notes_of(advance(line, semicolon + 1)) {
            if note == "pg_primary_key" field.primary_key = true;
            else if note == "pg_default" field.has_default = true;
            else if note == "pg_generated" field.generated = true;
            else if note == "pg_json" field.json_text = true;
        }

        array_add(*fields, field);
    }

    declared.fields = fields;

    return declared, "";
}

// `Member_Role` as `member_role`.
snake_case :: (name: string) -> string {
    return to_lower_copy(name,, temp);
}

// Spaces and the module prefix taken out, so `Pg.Nullable( Pg.Uuid )` and `Nullable(Uuid)` agree.
normal_type :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for text if it != #char " " append(*builder, slice(text, it_index, 1));

    out := builder_to_string(*builder,, temp);

    // `[]T` and `[] T` are alike; drop each `Pg.`, which is only ever a prefix of a name.
    while true {
        at := find_index_from_left(out, "Pg.");
        if at < 0 break;
        if at > 0 && (is_alnum(out[at - 1]) || out[at - 1] == #char "_") break;

        out = tprint("%%", slice(out, 0, at), advance(out, at + 3));
    }

    return out;
}

// The type without a `Nullable(...)` wrapper, and whether there was one.
unwrap_nullable :: (text: string) -> string, bool {
    type := normal_type(text);

    if begins_with(type, "Nullable(") && ends_with(type, ")") return slice(type, 9, type.count - 10), true;

    return type, false;
}

quote :: (name: string) -> string {
    return tprint("\"%\"", name);
}

qualified :: (schema: string, name: string) -> string {
    return tprint("\"%\".\"%\"", schema, name);
}

sql_literal :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    append(*builder, "'");

    for text {
        if it == #char "'" append(*builder, "'");
        append(*builder, slice(text, it_index, 1));
    }

    append(*builder, "'");

    return builder_to_string(*builder,, temp);
}

// The Postgres name of every enum of the file: the catalog's when it matches by Jai name or by
// schema and name, else the one the file implies.
enum_refs :: (catalog: Catalog, names: Names, description: Schema_Description) -> [] Enum_Ref {
    refs: [..] Enum_Ref;
    refs.allocator = temp;

    for description.enums {
        ref := Enum_Ref.{ it.type_name, it.schema, it.name };

        for catalog.enums {
            if names.enums[it_index] == ref.type_name {
                ref.schema = it.schema;
                ref.name = it.name;
            }
        }

        array_add(*refs, ref);
    }

    return refs;
}

line_todo :: (p: *Proposal_Builder, text: string) {
    print_to_builder(*p.notes, "-- TODO: %\n", text);
    p.todos += 1;
}

// A commented-out statement, with the warning or hint in front of it.
line_comment_statement :: (p: *Proposal_Builder, text: string) {
    print_to_builder(*p.notes, "-- %\n", text);
}

statement :: (p: *Proposal_Builder, builder: *String_Builder, text: string) {
    print_to_builder(builder, "%\n", text);
    p.changes += 1;
}

// `create schema` for a schema nothing in the catalog has.
need_schema :: (p: *Proposal_Builder, schema: string) {
    if schema == "public" || array_find(p.new_schemas, schema) return;

    for p.catalog.relations if it.schema == schema return;
    for p.catalog.enums if it.schema == schema return;

    array_add(*p.new_schemas, schema);
    statement(p, *p.schemas, tprint("create schema if not exists %;", quote(schema)));
}

propose_enum :: (p: *Proposal_Builder, catalog: Catalog, names: Names, declared: Schema_Enum) {
    existing := -1;

    for catalog.enums {
        if names.enums[it_index] == declared.type_name || (it.schema == declared.schema && it.name == declared.name) existing = it_index;
    }

    if existing < 0 {
        need_schema(p, declared.schema);

        builder: String_Builder;
        builder.allocator = temp;

        for declared.labels print_to_builder(*builder, "%%", ifx it_index then ", " else "", sql_literal(it));

        statement(p, *p.enums, tprint("create type % as enum (%);", qualified(declared.schema, declared.name), builder_to_string(*builder,, temp)));

        return;
    }

    old := catalog.enums[existing];
    current: [..] string;
    current.allocator = temp;
    for old.labels array_add(*current, it);

    last := ifx old.labels.count then old.labels[old.labels.count - 1] else "";

    for label: declared.labels {
        if array_find(current, label) continue;

        position := "";

        if it_index > 0 {
            previous := declared.labels[it_index - 1];

            if previous != last position = tprint(" after %", sql_literal(previous));
        } else {
            for later: declared.labels if array_find(current, later) {
                position = tprint(" before %", sql_literal(later));
                break;
            }
        }

        statement(p, *p.enums, tprint("alter type % add value %%;", qualified(old.schema, old.name), sql_literal(label), position));

        if !position last = label;

        array_add(*current, label);
    }

    for old.labels {
        if !array_find(declared.labels, it) {
            line_todo(p, tprint("the label % of % is no longer in the enum %; Postgres cannot drop an enum value, so rebuild the type by hand if you need to", sql_literal(it), qualified(old.schema, old.name), declared.type_name));
        }
    }
}

// The column definition of a field: type and `not null`.
column_definition :: (p: *Proposal_Builder, owner: string, field: Schema_Field) -> string {
    inner, nullable := unwrap_nullable(field.type_text);
    sql := sql_type_of_text(inner, p.refs);

    if sql == "jsonb" && field.json_text sql = "json";

    if !sql {
        line_todo(p, tprint("%.%: no Postgres type for the Jai type %; the column is left out", owner, quote(field.name), field.type_text));
        return "";
    }

    if field.has_default line_todo(p, tprint("%.%: set the default (@pg_default says there is one)", owner, quote(field.name)));
    if field.generated line_todo(p, tprint("%.%: make it a generated column (@pg_generated)", owner, quote(field.name)));

    return tprint("% %%", quote(field.name), sql, ifx nullable then "" else " not null");
}

propose_create :: (p: *Proposal_Builder, declared: Schema_Struct) {
    need_schema(p, declared.schema);

    owner := qualified(declared.schema, declared.table);
    lines: [..] string;
    lines.allocator = temp;

    keys: [..] string;
    keys.allocator = temp;

    for declared.fields {
        definition := column_definition(p, owner, it);
        if !definition continue;

        array_add(*lines, definition);
        if it.primary_key array_add(*keys, quote(it.name));
    }

    if keys.count array_add(*lines, tprint("primary key (%)", join(..keys, separator = ", ",, temp)));

    for declared.belongs_to {
        parts := split(it, ":");
        target: *Schema_Struct;

        if parts.count == 4 for candidate: p.description.structs if candidate.type_name == parts[2] target = *p.description.structs[it_index];

        if !target {
            line_todo(p, tprint("%: add the foreign key for @pg_belongs_to(%) by hand", owner, it));
            continue;
        }

        own := split(parts[1], ",");
        theirs := split(parts[3], ",");

        array_add(*lines, tprint("foreign key (%) references % (%)", quote_all(own), qualified(target.schema, target.table), quote_all(theirs)));
    }

    builder: String_Builder;
    builder.allocator = temp;

    print_to_builder(*builder, "create table % (\n", owner);
    for lines print_to_builder(*builder, "    %%\n", it, ifx it_index < lines.count - 1 then "," else "");

    append(*builder, ");\n");

    statement(p, *p.creates, builder_to_string(*builder,, temp));
}

quote_all :: (names: [] string) -> string {
    quoted := NewArray(names.count, string,, temp);
    for names quoted[it_index] = quote(it);

    return join(..quoted, separator = ", ",, temp);
}

propose_alter :: (p: *Proposal_Builder, relation: Catalog_Relation, declared: Schema_Struct) {
    owner := qualified(relation.schema, relation.name);
    added: [..] string;
    removed: [..] string;

    added.allocator = temp;
    removed.allocator = temp;

    for declared.fields {
        index := -1;
        for column: relation.columns if column.name == it.name index = it_index;

        if index < 0 {
            definition := column_definition(p, owner, it);
            if !definition continue;

            _, nullable := unwrap_nullable(it.type_text);

            if !nullable && !it.has_default {
                line_todo(p, tprint("%.%: not null with no default fails on a table that has rows; add a default or fill it in first", owner, quote(it.name)));
            }

            statement(p, *p.alters, tprint("alter table % add column %;", owner, definition));
            array_add(*added, it.name);
            continue;
        }

        compare_column(p, relation, relation.columns[index], it, owner);
    }

    for relation.columns {
        found := false;
        for field: declared.fields if field.name == it.name found = true;
        if found continue;

        array_add(*removed, it.name);
        line_comment_statement(p, tprint("WARNING: dropping a column destroys its data.\n-- alter table % drop column %;", owner, quote(it.name)));
    }

    if added.count && removed.count {
        line_todo(p, tprint("%: % added and % removed; if a column was renamed, use `alter table ... rename column ... to ...` instead", owner, join(..added, separator = ", ",, temp), join(..removed, separator = ", ",, temp)));
    }
}

// A column both sides have: a changed type or nullability.
compare_column :: (p: *Proposal_Builder, relation: Catalog_Relation, column: Catalog_Column, field: Schema_Field, owner: string) {
    inner, nullable := unwrap_nullable(field.type_text);

    // The Jai type the generator would write for the catalog's column.
    plain := column;
    plain.nullable = false;

    expected, problem := field_type(relation, plain, p.catalog, p.names);
    if problem return;

    // A json column's Jai type comes from its `Json_Columns` mapping, not from the database: any
    // `Json(...)` on both sides is the same column type.
    both_json := begins_with(normal_type(expected), "Json(") && begins_with(inner, "Json(");

    if normal_type(expected) != inner && !both_json {
        sql := sql_type_of_text(inner, p.refs);

        line_todo(p, tprint("%.%: the type changed from % to %; write the conversion by hand, such as `alter table % alter column % type % using ...`", owner, quote(column.name), expected, inner, owner, quote(column.name), ifx sql then sql else "..."));
    }

    is_json := column.base == "json" || column.base == "jsonb";

    if is_json && field.json_text != (column.base == "json") {
        to := ifx field.json_text then "json" else "jsonb";
        statement(p, *p.alters, tprint("alter table % alter column % type % using %::%;", owner, quote(column.name), to, quote(column.name), to));
    }

    if nullable && !column.nullable {
        statement(p, *p.alters, tprint("alter table % alter column % drop not null;", owner, quote(column.name)));
    } else if !nullable && column.nullable {
        line_todo(p, tprint("%.%: now not null; this fails while it holds nulls, so fill them in first", owner, quote(column.name)));
        line_comment_statement(p, tprint("alter table % alter column % set not null;", owner, quote(column.name)));
    }
}
