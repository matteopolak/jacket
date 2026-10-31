// The ORM: `select`, `update`, `delete` and `insert` on the table structs of db/schema.jai. Each
// is a macro that turns its filters (Jai expressions in `Code`) into SQL while the program
// compiles and hands the statement to the typed queries (typed.jai), so it is described, checked
// and cached like any `Pg.query`. Nothing is built at run time.
//
//     min_score := 3;
//     users, error := Pg.select(db, User, where = #code it.active && it.score >= min_score, order_by = #code it.email, limit = 20);
//
// In a filter, `it.<field>` is a column, literals stay literals, and any other expression is a
// parameter, evaluated where the filter was written. See docs/postgres/orm.md.

// Defaults for `select`: no filter, no order.
EVERY_ROW :: #code true;
NO_ORDER :: #code null;

// The rows of `T`'s table that pass `where`, in `order_by` order (`desc` for descending), at most
// `limit` of them (-1: all) after skipping `offset`. `from_sql` replaces the table with a query of
// your own that returns its columns, with `%name` / `%(expression)` parameters:
// `from_sql = "select * from users_visible_to(%viewer)"`. `fields = "id, email"` selects only
// those columns, and the rows are then `Partial(T, "id, email")`.
//
// - db: a connection or pool.
// - T: the table struct.
// - where: the filter. Every row by default.
// - order_by: the sort key. No order by default.
// - desc: sort descending.
// - limit: the most rows to return. -1 for all.
// - offset: how many rows to skip first.
// - from_sql: a query to select from instead of the table.
// - fields: the comma-separated columns to select. All by default.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
select :: (db: $D, $T: Type, $where: Code = EVERY_ROW, $order_by: Code = NO_ORDER, $desc := false, $limit := -1, $offset := 0, $from_sql := "", $fields := "", $call := #caller_code) -> Rows(#insert #run orm_select_row(fields)), Pg_Error #expand {
    #insert #run orm_statement_code(T, .SELECT, where, order_by, where, desc, limit, offset, from_sql, fields, call);
}

// Some fields of a table struct, with their types and notes: the rows of `select(..., fields = ...)`.
// `fields` is a comma-separated list of `T`'s field names.
//
// ```jai,skip_on(wasm)
// Contact :: Pg.Partial(User, "id, email");
// ```
Partial :: struct(T: Type, fields: string) {
    #insert #run orm_partial_body(T, fields);
}

// Sets fields of the rows that pass `where`: `set = #code it.score = score + 1` (or a block of such
// assignments). Returns the number of rows changed.
//
// - db: a connection or pool.
// - T: the table struct.
// - set: the assignments to make.
// - where: the filter.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
update :: (db: $D, $T: Type, $set: Code, $where: Code, $call := #caller_code) -> s64, Pg_Error #expand {
    #insert #run orm_statement_code(T, .UPDATE, where, where, set, false, -1, 0, "", "", call);
}

// Deletes the rows that pass `where` and returns how many there were. `where` is required: pass
// `#code true` to delete every row.
//
// - db: a connection or pool.
// - T: the table struct.
// - where: the filter.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
delete :: (db: $D, $T: Type, $where: Code, $call := #caller_code) -> s64, Pg_Error #expand {
    #insert #run orm_statement_code(T, .DELETE, where, where, where, false, -1, 0, "", "", call);
}

// Inserts `value` and returns the row the table now holds (with its defaults and generated
// columns). Fields with a database default (`@pg_default`) or a generated value
// (`@pg_generated`) are left to the database.
//
// - db: a connection or pool.
// - value: the row to insert.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
insert :: (db: $D, value: $T, $call := #caller_code) -> T, Pg_Error #expand {
    #insert #run orm_insert_code(T, call);
}

// Inserts each value in turn (one statement each), stopping at the first error. Returns the rows
// inserted so far, from context.allocator.
//
// - db: a connection or pool.
// - values: the rows to insert.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
insert_many :: (db: $D, values: [] $T, $call := #caller_code) -> [] T, Pg_Error {
    rows: [..] T;

    for values {
        row, error := insert(db, it, call = call);
        if error.kind != .NONE return rows, error;

        array_add(*rows, row);
    }

    return rows, .{};
}

// A select whose columns come from the loop that reads it: only the fields of `T` the body reads,
// plus the primary key, are fetched; the other fields of `it` stay zero. A json document read only
// for some top-level fields comes back with just those keys, and `it.author.email` (a to-one
// relation) is fetched in the same statement, making `it` an `Orm_Item` that `using`s a `T`. When `it` is used whole
// (passed to a procedure, stored, `*it`), every column is fetched. The query runs when the loop
// starts, and a failure lands in `error` (which is cleared first). Strings in `it` read from the
// result, which is freed when the loop ends: copy what you keep.
//
// - db: a connection or pool.
// - T: the table struct.
// - error: set to the failure, if any, once the loop starts.
// - where: the filter. Every row by default.
// - order_by: the sort key. No order by default.
// - desc: sort descending.
// - limit: the most rows to read. -1 for all.
// - offset: how many rows to skip first.
// - from_sql: a query to select from instead of the table.
// - strategy: how to load has-many relations the body reads.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
//
// ```jai,ignore
// error: Pg.Pg_Error;
// for Pg.each(db, User, *error, where = #code it.score > min) print("%\n", it.email);   // select "id", "email" ...
// ```
//
// Not run: it needs a PostgreSQL server.
each :: (db: $D, $T: Type, error: *Pg_Error, $where: Code = EVERY_ROW, $order_by: Code = NO_ORDER, $desc := false, $limit := -1, $offset := 0, $from_sql := "", $strategy := Orm_Strategy.PRELOAD, $call := #caller_code) -> Orm_Each(D, T, where, order_by, desc, limit, offset, from_sql, strategy, call) {
    if error error.* = .{};
    return .{ db = db, error = error };
}

// How `Pg.each` loads a has-many relation its loop body reads (`for post: it.posts`).
Orm_Strategy :: enum u8 {
    PRELOAD; // a second query for every row's related rows, as `load_many` does
    LATERAL; // one query: each related column comes back as an array from a lateral join
}

Orm_Each :: struct(D: Type, T: Type, where: Code, order_by: Code, desc: bool, limit: s64, offset: s64, from_sql: string, strategy: Orm_Strategy, call: Code) {
    db: D;
    error: *Pg_Error;
}

// Makes `Pg.each` loopable.
//
// - q: the `Pg.each` call.
// - body: the loop body.
// - flags: the loop's flags.
for_expansion :: (q: *Orm_Each($D, $T, $where, $order_by, $desc, $limit, $offset, $from_sql, $strategy, $call), body: Code, flags: For_Flags) #expand {
    #insert #run orm_each_code(T, where, order_by, desc, limit, offset, from_sql, strategy, body, call);
}

// Stores `values` with one binary COPY and returns how many rows the server stored. Much faster
// than `insert_many` for many rows, but it reads nothing back: fields marked `@pg_default` or
// `@pg_generated` are left to the database, as in `insert`, and their values are not returned.
//
// - db: a connection or pool.
// - values: the rows to store.
copy_many :: (db: $D, values: [] $T) -> s64, Pg_Error {
    #insert #run orm_copy_code(T);
}

// The rows `parents` have through the has-many relation `relation` (a `@pg_has_many` note of
// `T`), in one query: `items[i]` belongs to `parents[i]`. Free it with `free_loaded`; the items
// read from its result.
//
// ```jai,ignore
// posts, error := Pg.load_many(db, users, "posts");
// for users print("% wrote % posts\n", it.email, posts.items[it_index].count);
// ```
//
// Not run: it needs a PostgreSQL server.
Loaded_Many :: struct(Item: Type) {
    items: [][] Item;
    // Every item once; `items` are views into it.
    rows: [] Item;
    result: Result;
}

// The row each of `parents` points to through the to-one relation `relation` (a
// `@pg_belongs_to` note of `T`), in one query: `items[i]` is null when `parents[i]` points nowhere.
Loaded_One :: struct(Item: Type) {
    items: [] Nullable(Item);
    rows: [] Item;
    result: Result;
}

// Loads a has-many relation for several rows; see `Loaded_Many`.
//
// - db: a connection or pool.
// - parents: the rows to load for.
// - relation: the name of the `@pg_has_many` relation.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
load_many :: (db: $D, parents: [] $T, $relation: string, $call := #caller_code) -> Loaded_Many(#insert,scope(call) #run orm_relation_target(T, relation, .HAS_MANY)), Pg_Error #expand {
    #insert #run orm_load_code(T, relation, .HAS_MANY, "", orm_project_argument(call));
}

// Loads a to-one relation for several rows; see `Loaded_One`.
//
// - db: a connection or pool.
// - parents: the rows to load for.
// - relation: the name of the `@pg_belongs_to` relation.
// - call: the code of the call, which lets the query text be checked while compiling. Leave the default.
load_one :: (db: $D, parents: [] $T, $relation: string, $call := #caller_code) -> Loaded_One(#insert,scope(call) #run orm_relation_target(T, relation, .BELONGS_TO)), Pg_Error #expand {
    #insert #run orm_load_code(T, relation, .BELONGS_TO, "", orm_project_argument(call));
}

// `load_many` with the related struct given, for `Pg.each`, whose loop knows it already.
//
// - db: a connection or pool.
// - parents: the rows to load for.
// - relation: the name of the `@pg_has_many` relation.
// - Target: the related struct.
// - project_file: the file the loop is in.
orm_load_many_as :: (db: $D, parents: [] $T, $relation: string, $Target: Type, $project_file: string) -> Loaded_Many(Target), Pg_Error {
    #insert #run orm_load_code(T, relation, .HAS_MANY, "Target", ", project_file = project_file");
}

// Frees what `load_many` returned.
//
// - loaded: the loaded rows. They are zeroed.
free_loaded :: (loaded: *Loaded_Many($Item)) {
    array_free(loaded.items);
    array_free(loaded.rows);
    free_result(*loaded.result);
    loaded.* = .{};
}

// Frees what `load_one` returned.
//
// - loaded: the loaded rows. They are zeroed.
free_loaded :: (loaded: *Loaded_One($Item)) {
    array_free(loaded.items);
    array_free(loaded.rows);
    free_result(*loaded.result);
    loaded.* = .{};
}

// What the filters may call; they only mean something inside `where`, as SQL. Two more have no
// procedure here, because their names are common ones:
//
// - `Pg.contains(it.settings, value)`: `@>`, the jsonb column (or a document inside it) contains
//   `value`, sent as json.
// - `Pg.sql("lower(%(it.email)) = lower(%email)")`: raw SQL, with `%(it.field)` for a column,
//   `%name` / `%(expression)` for a parameter and `%%` for a percent sign. The text must be a
//   string literal.

// `like`: `Pg.like(it.email, "%@example.com")`.
//
// - value: the text to test.
// - pattern: the `like` pattern.
like :: (value: string, pattern: string) -> bool {
    return false;
}

// `ilike`: case-insensitive `like`.
//
// - value: the text to test.
// - pattern: the `like` pattern.
ilike :: (value: string, pattern: string) -> bool {
    return false;
}

// `= any(values)`: `Pg.in(it.id, ids)`.
//
// - value: the value to look for.
// - values: the values it may be.
in :: (value: $T, values: [] T) -> bool {
    return false;
}

// `lower(value)`.
//
// - value: the text.
lower :: (value: string) -> string {
    return "";
}

// `upper(value)`.
//
// - value: the text.
upper :: (value: string) -> string {
    return "";
}

// `(value is null or condition)`: an optional filter, `Pg.if_some(team, #code it.team_id == team)`
// with `team: Nullable(s64)`, where `team` stands for its value inside the condition.
//
// - value: the optional value.
// - condition: the filter used when `value` is set.
if_some :: (value: $T, condition: Code) -> bool {
    return false;
}

// `?`: the jsonb document has the top-level key: `Pg.has_key(it.settings, "beta")`.
//
// - document: the jsonb column.
// - key: the top-level key.
has_key :: (document: $T, key: string) -> bool {
    return false;
}

// `@?`: the jsonpath matches: `Pg.json_path(it.settings, "$.tags[*] ? (@ == \"x\")")`.
//
// - document: the jsonb column.
// - path: the jsonpath.
json_path :: (document: $T, path: string) -> bool {
    return false;
}

#scope_module

Orm_Print :: #import "Program_Print";
Orm_Hash :: #import "Hash_Table"; // jailint: allow(unused_import) the code load_many and load_one generate uses it

Orm_Kind :: enum {
    SELECT;
    UPDATE;
    DELETE;
}

// The table behind a struct of db/schema.jai: `"schema"."table"`.
orm_table :: (T: Type) -> string, problem: string {
    info := cast(*Type_Info_Struct) type_info(T);

    if info.type != .STRUCT return "", tprint("% is not a struct", T);

    for info.notes {
        name, found := note_arguments(it, "pg_table");
        if !found continue;

        dot := find_index_from_left(name, #char ".");

        if dot < 0 return quote_identifier(name), "";

        return tprint("%.%", quote_identifier(slice(name, 0, dot)), quote_identifier(advance(name, dot + 1))), "";
    }

    return "", tprint("% has no @pg_table note: the ORM works on the structs of db/schema.jai", T);
}

orm_columns :: (T: Type) -> [] *Type_Info_Struct_Member {
    info := cast(*Type_Info_Struct) type_info(T);
    columns: [..] *Type_Info_Struct_Member;
    columns.allocator = temp;

    for *info.members {
        if (it.flags & .CONSTANT) || (it.flags & .USING) continue;

        array_add(*columns, it);
    }

    return columns;
}

// What is inside the parentheses of a note `name(...)`.
note_arguments :: (note: string, name: string) -> string, found: bool {
    if note.count < name.count + 2 || !begins_with(note, name) || note[name.count] != #char "(" || !ends_with(note, ")") return "", false;

    return slice(note, name.count + 1, note.count - name.count - 2), true;
}

has_note :: (member: *Type_Info_Struct_Member, note: string) -> bool {
    for member.notes if it == note return true;
    return false;
}

// A Jai string literal holding `text`.
jai_string :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "\"");

    for text {
        if it == #char "\"" append(*builder, "\\\"");
        else if it == #char "\\" append(*builder, "\\\\");
        else if it == #char "\n" append(*builder, "\\n");
        else append(*builder, it);
    }

    append(*builder, "\"");

    return builder_to_string(*builder);
}

// The SQL a filter becomes, with a `%` per parameter and each parameter's Jai text.
Orm_Sql :: struct {
    T: Type;
    statement_type: Type; // `T` of the statement: `T` changes inside a relation's subquery
    builder: String_Builder;
    parameters: [..] string; // as Jai of the generated code (see `orm_argument`)
    // Per parameter: a procedure of this module the value goes through first, or "".
    wraps: [..] string;
    // Set by the second pass: each parameter's Jai type, for the cast after its hole.
    parameter_types: [] Type;
    problem: string;
    problem_node: *Code_Node;
    // Inside `if_some`: the Nullable whose value the condition reads.
    some_name: string;
    // How columns of the statement's own table are qualified inside a relation's subquery.
    outer: string;
    // The struct names of the to-one relations the filter reads (first pass), and their types
    // (second pass), in the same order.
    relation_names: [..] string;
    relation_types: [] Type;
    // Subqueries opened so far, for their aliases.
    subqueries: s64;
    // `left join lateral ...` clauses for has-many relations a loop reads in one query.
    joins: [..] string;
}

orm_fail :: (sql: *Orm_Sql, node: *Code_Node, message: string) {
    if sql.problem return;

    sql.problem = message;
    sql.problem_node = node;
}

orm_root :: (code: Code) -> *Code_Node {
    root := compiler_get_nodes(code);

    while root && root.kind == .DIRECTIVE_CODE root = (cast(*Code_Directive_Code) root).expression;

    return root;
}

is_null_code :: (node: *Code_Node) -> bool {
    if !node return true;
    if node.kind != .IDENT && node.kind != .LITERAL return false;
    if node.kind == .IDENT return (cast(*Code_Ident) node).name == "null";

    literal := cast(*Code_Literal) node;

    return literal.value_type == .POINTER;
}

is_ident :: (node: *Code_Node, name: string) -> bool {
    return node && node.kind == .IDENT && (cast(*Code_Ident) node).name == name;
}

// Whether `node` reads `it` anywhere.
mentions_it :: (node: *Code_Node) -> bool {
    if !node return false;

    if node.kind == {
        case .IDENT;
            return (cast(*Code_Ident) node).name == "it";

        case .BINARY_OPERATOR;
            binary := cast(*Code_Binary_Operator) node;
            if binary.operator_type == #char "." return mentions_it(binary.left);

            return mentions_it(binary.left) || mentions_it(binary.right);

        case .UNARY_OPERATOR;
            return mentions_it((cast(*Code_Unary_Operator) node).subexpression);

        case .PROCEDURE_CALL;
            call := cast(*Code_Procedure_Call) node;
            for call.arguments_unsorted if mentions_it(it.expression) return true;

            return false;

        case .CAST;
            return mentions_it((cast(*Code_Cast) node).expression);

        case .DIRECTIVE_CODE;
            return mentions_it((cast(*Code_Directive_Code) node).expression);
    }

    return false;
}

// The member chain `it.a.b`, or an empty array when `node` is not one.
field_path :: (node: *Code_Node) -> [] string {
    path: [..] string;
    path.allocator = temp;

    while node && node.kind == .BINARY_OPERATOR {
        binary := cast(*Code_Binary_Operator) node;
        if binary.operator_type != #char "." || binary.right.kind != .IDENT return .[];

        array_insert_at(*path, (cast(*Code_Ident) binary.right).name, 0);
        node = binary.left;
    }

    if !is_ident(node, "it") return .[];

    return path;
}

// The name a filter calls, `Pg.like` or `like`.
called_name :: (call: *Code_Procedure_Call) -> string {
    expression := call.procedure_expression;

    if expression.kind == .IDENT return (cast(*Code_Ident) expression).name;

    if expression.kind == .BINARY_OPERATOR {
        binary := cast(*Code_Binary_Operator) expression;
        if binary.operator_type == #char "." && binary.right.kind == .IDENT return (cast(*Code_Ident) binary.right).name;
    }

    return "";
}

call_argument :: (call: *Code_Procedure_Call, index: s64) -> *Code_Node {
    if index >= call.arguments_unsorted.count return null;
    return call.arguments_unsorted[index].expression;
}

member_of :: (T: Type, name: string) -> *Type_Info_Struct_Member {
    for orm_columns(T) if it.name == name return it;
    return null;
}

// Appends a parameter hole for the Jai expression `node`, passed through `wrap` when it is set.
orm_parameter :: (sql: *Orm_Sql, node: *Code_Node, wrap := "") {
    builder: String_Builder;
    builder.allocator = temp;

    if !Orm_Print.print_expression(*builder, node) {
        orm_fail(sql, node, "the ORM cannot pass this expression as a parameter; compute it into a variable first");
        return;
    }

    orm_hole(sql, builder_to_string(*builder), wrap);
}

// The struct a relation leads to, as text to insert where the call was written.
orm_relation_target :: (T: Type, name: string, kind: Orm_Relation_Kind) -> string {
    relation, problem := orm_relation(T, name);

    if !problem && relation.kind == .NONE problem = tprint("% has no relation \"%\"", T, name);

    if !problem && relation.kind != kind {
        problem = tprint("%.% is a % relation: load it with Pg.%", T, name, ifx relation.kind == .HAS_MANY then "has-many" else "to-one", ifx relation.kind == .HAS_MANY then "load_many" else "load_one");
    }

    if !problem && relation.own_columns.count != relation.target_columns.count problem = tprint("%.%: the relation note pairs % columns with %", T, name, relation.own_columns.count, relation.target_columns.count);

    if problem {
        compiler_report(problem);
        return "void";
    }

    return relation.target;
}

orm_value_type :: (T: Type) -> Type {
    info := type_info(T);
    if is_nullable_info(info) return get_type(nullable_value_type(info));

    return T;
}

orm_is_nullable :: (T: Type) -> bool {
    return is_nullable_info(type_info(T));
}

// `select <every column of Target> from <its table> where "<column>" = any($1)`, or for a key on
// several columns `where ("a", "b") in (select * from unnest($1, $2))`, one array per column.
orm_load_sql :: (Target: Type, columns: string, keys: [] Type) -> string {
    table, problem := orm_table(Target);

    if problem {
        compiler_report(problem);
        return "";
    }

    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "select ");

    for orm_columns(Target) {
        if it_index append(*builder, ", ");
        append(*builder, quote_identifier(it.name));
    }

    print_to_builder(*builder, " from % where ", table);

    names := split(columns, ",",, temp);

    if names.count == 1 {
        print_to_builder(*builder, "% = any(\%%)", quote_identifier(names[0]), orm_array_cast(keys[0]));
        return builder_to_string(*builder);
    }

    append(*builder, "(");

    for names {
        if it_index append(*builder, ", ");
        append(*builder, quote_identifier(it));
    }

    append(*builder, ") in (select * from unnest(");

    for keys {
        if it_index append(*builder, ", ");
        print_to_builder(*builder, "\%%", orm_array_cast(it));
    }

    append(*builder, "))");

    return builder_to_string(*builder);
}

orm_array_cast :: (Key: Type) -> string {
    cast_to := parameter_cast(Key);
    return ifx cast_to then tprint(" ::%[]", cast_to) else "";
}

// One part of a key on several columns, length-prefixed so parts cannot run together.
orm_key_part :: (builder: *String_Builder, value: $V) {
    text := tprint("%", value);
    print_to_builder(builder, "%:%;", text.count, text);
}

// The body of `load_many` and `load_one`: collect the keys, run one query, and hand each row to
// the parents whose key it carries. A key on one column is that column's value; on several, a
// string made of the parts. A parent or row with a null part has no key and matches nothing.
// `project` is the `project_file` argument of the query (see `orm_project_argument`).
orm_load_code :: (T: Type, name: string, kind: Orm_Relation_Kind, target: string, project: string) -> string {
    relation, problem := orm_relation(T, name);
    if problem || relation.kind != kind || relation.own_columns.count != relation.target_columns.count return "";

    own_nullable: [..] bool;
    own_nullable.allocator = temp;

    for relation.own_columns {
        member := member_of(T, it);

        if !member {
            compiler_report(tprint("% has no field \"%\" for its relation \"%\"", T, it, name));
            return "";
        }

        array_add(*own_nullable, is_nullable_info(member.type));
    }

    single := relation.own_columns.count == 1;
    builder: String_Builder;
    builder.allocator = temp;

    print_to_builder(*builder, "Orm_Target :: %;\n", ifx target then target else orm_insert_text(relation.target));

    for relation.own_columns print_to_builder(*builder, "Orm_K% :: #run orm_value_type(type_of(parents[0].%));\n", it_index, it);

    append(*builder, "Orm_Key_Types :: Type.[");
    for relation.own_columns print_to_builder(*builder, "%Orm_K%", ifx it_index then ", " else "", it_index);

    append(*builder, "];\n");

    print_to_builder(*builder, "Orm_Key :: %;\n", ifx single then "Orm_K0" else "string");

    // The parent's key, and false when a part of it is null.
    append(*builder, "orm_parent_key :: (it: T) -> Orm_Key, bool {\n    key: Orm_Key;\n");

    for relation.own_columns if own_nullable[it_index] print_to_builder(*builder, "    if it.%.null return key, false;\n", it);

    if single {
        print_to_builder(*builder, "    return it.%1%2, true;\n", relation.own_columns[0], ifx own_nullable[0] then ".value" else "");
    } else {
        append(*builder, "    builder: String_Builder;\n    builder.allocator = temp;\n");
        for relation.own_columns print_to_builder(*builder, "    orm_key_part(*builder, it.%1%2);\n", it, ifx own_nullable[it_index] then ".value" else "");
        append(*builder, "    return builder_to_string(*builder), true;\n");
    }

    append(*builder, "}\n");

    // The same for a fetched row, through the related struct's columns (nullable or not).
    append(*builder, "orm_child_key :: (row: Orm_Target) -> Orm_Key, bool {\n    key: Orm_Key;\n");

    for relation.target_columns print_to_builder(*builder, "    #if orm_is_nullable(type_of(row.%1)) if row.%1.null return key, false;\n", it);

    if single {
        print_to_builder(*builder, "    #if orm_is_nullable(type_of(row.%1)) return row.%1.value, true; else return row.%1, true;\n", relation.target_columns[0]);
    } else {
        append(*builder, "    builder: String_Builder;\n    builder.allocator = temp;\n");
        for relation.target_columns print_to_builder(*builder, "    #if orm_is_nullable(type_of(row.%1)) orm_key_part(*builder, row.%1.value); else orm_key_part(*builder, row.%1);\n", it);
        append(*builder, "    return builder_to_string(*builder), true;\n");
    }

    append(*builder, "}\n");

    print_to_builder(*builder, "orm_loaded: %(Orm_Target);\n", ifx kind == .HAS_MANY then "Loaded_Many" else "Loaded_One");

    for relation.own_columns print_to_builder(*builder, "orm_keys_%1: [..] Orm_K%1;\norm_keys_%1.allocator = temp;\n", it_index);

    // One array per key column, each parent with a whole key adding its parts.
    append(*builder, "for parents {\n");
    for relation.own_columns if own_nullable[it_index] print_to_builder(*builder, "    if it.%.null continue;\n", it);
    for relation.own_columns print_to_builder(*builder, "    array_add(*orm_keys_%1, it.%2%3);\n", it_index, it, ifx own_nullable[it_index] then ".value" else "");

    append(*builder, "}\n");

    print_to_builder(*builder, "orm_rows, orm_error := query_as(Orm_Target, db, #run orm_load_sql(Orm_Target, %, Orm_Key_Types)", jai_string(join(..relation.target_columns, ",",, temp)));
    for relation.own_columns print_to_builder(*builder, ", cast([] Orm_K%1) orm_keys_%1", it_index);

    append(*builder, project);

    append(*builder, ");\n");

    append(*builder, #string DONE
if orm_error.kind != .NONE {
    free_rows(*orm_rows);
    return orm_loaded, orm_error;
}
orm_found := NewArray(orm_rows.result.row_count, Orm_Target);
for * orm_found {
    orm_read_error := read_row(*orm_rows, it_index, it);
    if orm_read_error.kind != .NONE {
        array_free(orm_found);
        free_rows(*orm_rows);
        return orm_loaded, orm_read_error;
    }
}
orm_loaded.rows = orm_found;
orm_loaded.result = orm_rows.result;
DONE);

    if kind == .HAS_MANY {
        append(*builder, #string DONE
// Parent key to its first index; parents sharing a key share their items.
orm_parent: Orm_Hash.Table(Orm_Key, s64);
orm_parent.allocator = temp;
for parents {
    key, has := orm_parent_key(it);
    if has && !Orm_Hash.table_contains(*orm_parent, key) Orm_Hash.table_add(*orm_parent, key, it_index);
}
orm_counts := NewArray(parents.count, s64,, temp);
for orm_found {
    key, has := orm_child_key(it);
    if !has continue;
    index := Orm_Hash.table_find_pointer(*orm_parent, key);
    if index orm_counts[index.*] += 1;
}
// Group the rows by parent in place, so each parent's items are one slice of `rows`.
orm_starts := NewArray(parents.count, s64,, temp);
orm_next := 0;
for orm_counts {
    orm_starts[it_index] = orm_next;
    orm_next += it;
}
orm_grouped := NewArray(orm_next, Orm_Target);
orm_fill := array_copy(orm_starts,, temp);
for orm_found {
    key, has := orm_child_key(it);
    if !has continue;
    index := Orm_Hash.table_find_pointer(*orm_parent, key);
    if !index continue;
    orm_grouped[orm_fill[index.*]] = it;
    orm_fill[index.*] += 1;
}
array_free(orm_found);
orm_loaded.rows = orm_grouped;
orm_loaded.items = NewArray(parents.count, [] Orm_Target);
for parents {
    key, has := orm_parent_key(it);
    if !has continue;
    index := Orm_Hash.table_find_pointer(*orm_parent, key);
    orm_loaded.items[it_index] = array_view(orm_grouped, orm_starts[index.*], orm_counts[index.*]);
}
return orm_loaded, .{};
DONE);
    } else {
        append(*builder, #string DONE
orm_found_at: Orm_Hash.Table(Orm_Key, s64);
orm_found_at.allocator = temp;
for orm_found {
    key, has := orm_child_key(it);
    if has Orm_Hash.table_set(*orm_found_at, key, it_index);
}
orm_loaded.items = NewArray(parents.count, Nullable(Orm_Target));
for parents {
    orm_loaded.items[it_index].null = true;
    key, has := orm_parent_key(it);
    if !has continue;
    index := Orm_Hash.table_find_pointer(*orm_found_at, key);
    if index orm_loaded.items[it_index] = .{ value = orm_found[index.*] };
}
return orm_loaded, .{};
DONE);
    }

    return builder_to_string(*builder);
}

// A value sent as a json document (for `contains`).
orm_json_value :: (value: $V) -> Json(V) {
    return .{ value = value };
}

// `text` is Jai where the call was written, or with `in_module` Jai of the generated code itself.
orm_hole :: (sql: *Orm_Sql, text: string, wrap := "", in_module := false) {
    index := sql.parameters.count;

    array_add(*sql.parameters, ifx in_module then text else orm_insert_text(text));
    array_add(*sql.wraps, wrap);
    append(*sql.builder, "% ");

    // `%` alone takes the type Postgres infers; with the argument's Jai type known, cast to it so
    // an s64 compares with an int4 column.
    if index < sql.parameter_types.count {
        cast_to := parameter_cast(sql.parameter_types[index]);
        if cast_to print_to_builder(*sql.builder, "::% ", cast_to);
    }
}

// The Postgres type a parameter of type `T` is sent as, or "" to let the server infer it.
parameter_cast :: (T: Type) -> string {
    info := type_info(T);

    if is_nullable_info(info) info = nullable_value_type(info);

    if info.type == {
        case .BOOL; return "bool";
        case .STRING; return "text";
        case .FLOAT; return ifx info.runtime_size == 4 then "float4" else "float8";

        case .INTEGER;
            if info.runtime_size == 8 return "int8";
            if info.runtime_size == 4 return "int4";

            return "int2";

        case .ARRAY;
            element := (cast(*Type_Info_Array) info).element_type;
            if element == type_info(u8) return "bytea";

            inner := parameter_cast(get_type(element));
            if inner return tprint("%[]", inner);

            return "";
    }

    if info == type_info(Uuid) return "uuid";
    if info == type_info(Timestamp) return "timestamptz";
    if info == type_info(Local_Timestamp) return "timestamp";
    if info == type_info(Date) return "date";
    if info == type_info(Time) return "time";
    if info == type_info(Interval) return "interval";
    if info == type_info(Numeric) return "numeric";

    return "";
}

orm_expression :: (sql: *Orm_Sql, node: *Code_Node) {
    if sql.problem return;

    // A fragment names its columns inside its text.
    if node.kind == .PROCEDURE_CALL && called_name(cast(*Code_Procedure_Call) node) == "sql" {
        orm_call(sql, cast(*Code_Procedure_Call) node);
        return;
    }

    if !mentions_it(node) {
        if node.kind == .LITERAL {
            orm_literal(sql, cast(*Code_Literal) node);
            return;
        }

        if is_null_code(node) {
            append(*sql.builder, "null");
            return;
        }

        if sql.some_name && is_ident(node, sql.some_name) {
            orm_hole(sql, tprint("%.value", sql.some_name));
            return;
        }

        orm_parameter(sql, node);

        return;
    }

    if node.kind == {
        case .IDENT;
            orm_fail(sql, node, "`it` on its own has no SQL form: compare one of its fields");

        case .BINARY_OPERATOR;
            orm_binary(sql, cast(*Code_Binary_Operator) node);

        case .UNARY_OPERATOR;
            unary := cast(*Code_Unary_Operator) node;

            if unary.operator_type == #char "!" {
                append(*sql.builder, "not (");
                orm_expression(sql, unary.subexpression);
                append(*sql.builder, ")");
            } else if unary.operator_type == #char "-" {
                append(*sql.builder, "-(");
                orm_expression(sql, unary.subexpression);
                append(*sql.builder, ")");
            } else {
                orm_fail(sql, node, "the ORM has no SQL for this op");
            }

        case .PROCEDURE_CALL;
            orm_call(sql, cast(*Code_Procedure_Call) node);

        case;
            orm_fail(sql, node, "the ORM cannot turn this expression into SQL");
    }
}

orm_literal :: (sql: *Orm_Sql, literal: *Code_Literal) {
    if literal.value_type == {
        case .NUMBER;
            if literal.value_flags & .FLOAT print_to_builder(*sql.builder, "%", literal._float64);
            else print_to_builder(*sql.builder, "%", literal._s64);

        case .STRING;
            print_to_builder(*sql.builder, "'%'", replace(literal._string, "'", "''",, temp));

        case .BOOLEAN;
            append(*sql.builder, ifx literal._s64 then "true" else "false");

        case .POINTER;
            append(*sql.builder, "null");

        case;
            orm_parameter(sql, literal);
    }
}

orm_column :: (sql: *Orm_Sql, node: *Code_Node) {
    orm_column_path(sql, node, field_path(node));
}

// The column `path[0]`, or a key inside it for a longer path. `node` is where a problem is reported.
orm_column_path :: (sql: *Orm_Sql, node: *Code_Node, path: [] string, as_document := false) {
    if !path.count {
        orm_fail(sql, node, "the ORM cannot turn this expression into SQL");
        return;
    }

    member := member_of(sql.T, path[0]);

    if !member {
        orm_related_column(sql, node, path, as_document);
        return;
    }

    if path.count == 1 {
        append(*sql.builder, quote_identifier(path[0]));
        return;
    }

    orm_json_path(sql, node, member, path, as_document);
}

// `it.team.name`, through the to-one relation `team` (`@pg_belongs_to`): a subquery on the
// related table, `(select "name" from "public"."teams" as orm_r0 where orm_r0."id" = <outer>."team_id")`,
// which is null when there is no related row.
orm_related_column :: (sql: *Orm_Sql, node: *Code_Node, path: [] string, as_document: bool) {
    relation, problem := orm_relation(sql.T, path[0]);

    if problem {
        orm_fail(sql, node, problem);
        return;
    }

    if relation.kind == .NONE {
        orm_fail(sql, node, tprint("% has no field \"%\"", sql.T, path[0]));
        return;
    }

    if relation.kind == .HAS_MANY {
        orm_fail(sql, node, tprint("%.% is a has-many relation, which a filter cannot read; load it with Pg.load_many", sql.T, path[0]));
        return;
    }

    if path.count == 1 {
        orm_fail(sql, node, tprint("%.% is a relation: compare one of its fields", sql.T, path[0]));
        return;
    }

    index := array_find_index(sql.relation_names, relation.target);

    if index < 0 {
        array_add(*sql.relation_names, relation.target);
        index = sql.relation_names.count - 1;
    }

    // The first pass does not know the related struct yet; the text it writes is not used.
    if index >= sql.relation_types.count {
        append(*sql.builder, "null");
        return;
    }

    target := sql.relation_types[index];
    table, table_problem := orm_table(target);

    if table_problem {
        orm_fail(sql, node, table_problem);
        return;
    }

    alias := tprint("orm_r%", sql.subqueries);

    sql.subqueries += 1;

    saved_type, saved_outer := sql.T, sql.outer;

    sql.T, sql.outer = target, alias;

    append(*sql.builder, "(select ");
    orm_column_path(sql, node, array_view(path, 1), as_document);

    sql.T, sql.outer = saved_type, saved_outer;

    print_to_builder(*sql.builder, " from % as % where ", table, alias);

    for relation.target_columns {
        if it_index append(*sql.builder, " and ");
        print_to_builder(*sql.builder, "%.% = %.%", alias, quote_identifier(it), sql.outer, quote_identifier(relation.own_columns[it_index]));
    }

    append(*sql.builder, ")");
}

array_find_index :: (items: [] string, item: string) -> s64 {
    for items if it == item return it_index;
    return -1;
}

Orm_Relation_Kind :: enum u8 {
    NONE;
    BELONGS_TO;
    HAS_MANY;
}

// A relation note of a db/schema.jai struct: `own_columns` of this table match `target_columns`
// of the struct named `target`.
Orm_Relation :: struct {
    kind: Orm_Relation_Kind;
    target: string;
    own_columns: [] string;
    target_columns: [] string;
}

// `@pg_belongs_to(name:cols:Target:target cols)` or `@pg_has_many(name:Source:source cols:own cols)`
// named `name` on `T`; `kind` is NONE when there is none.
orm_relation :: (T: Type, name: string) -> Orm_Relation, problem: string {
    info := cast(*Type_Info_Struct) type_info(T);
    relation: Orm_Relation;

    for info.notes {
        kind := Orm_Relation_Kind.NONE;
        arguments, found := note_arguments(it, "pg_belongs_to");

        if found {
            kind = .BELONGS_TO;
        } else {
            arguments, found = note_arguments(it, "pg_has_many");
            if found kind = .HAS_MANY;
        }

        if kind == .NONE continue;

        parts := split(arguments, ":",, temp);
        if parts.count != 4 return relation, tprint("malformed relation note @%", it);
        if parts[0] != name continue;

        relation.kind = kind;

        if kind == .BELONGS_TO {
            relation.own_columns = split(parts[1], ",",, temp);
            relation.target = parts[2];
            relation.target_columns = split(parts[3], ",",, temp);
        } else {
            relation.target = parts[1];
            relation.target_columns = split(parts[2], ",",, temp);
            relation.own_columns = split(parts[3], ",",, temp);
        }

        if relation.own_columns.count != relation.target_columns.count return relation, tprint("malformed relation note @%", it);

        return relation, "";
    }

    return relation, "";
}

// `it.settings.theme`: a key of a `Json(T)` column, by `T`'s json names. A string leaf is read as
// text (`->>`); any other leaf is cast from its text. `as_document` keeps the leaf a jsonb
// document (`->` all the way), for the jsonb operators.
orm_json_path :: (sql: *Orm_Sql, node: *Code_Node, member: *Type_Info_Struct_Member, path: [] string, as_document := false) {
    keys, info, ok := orm_json_keys(sql, node, member, path);
    if !ok return;

    orm_json_access(sql, path[0], keys, as_document);
    if as_document return;

    leaf := info;
    if is_nullable_info(leaf) leaf = nullable_value_type(leaf);

    maybe := json_maybe_value(leaf);
    if maybe leaf = maybe;

    if leaf.type != .STRING {
        cast_to := parameter_cast(get_type(leaf));
        if leaf.type == .BOOL cast_to = "bool";

        if !cast_to {
            orm_fail(sql, node, tprint("a json % has no SQL comparison; compare one of its fields", get_type(leaf)));
            return;
        }

        print_to_builder(*sql.builder, "::%", cast_to);
    }
}

// The json keys of `path` after its column `path[0]`, and the type at its end. A `Json_Maybe`'s
// `value` is the key itself, so it adds no key.
orm_json_keys :: (sql: *Orm_Sql, node: *Code_Node, member: *Type_Info_Struct_Member, path: [] string) -> [] string, *Type_Info, bool {
    info := member.type;
    if is_nullable_info(info) info = nullable_value_type(info);

    if !is_json_info(info) {
        orm_fail(sql, node, tprint("%.% is not a json column, so it has no fields in SQL", sql.T, path[0]));
        return .[], null, false;
    }

    info = nullable_value_type(info);

    keys: [..] string;
    keys.allocator = temp;

    for 1..path.count - 1 {
        name := path[it];
        maybe := json_maybe_value(info);

        if maybe && name == "value" {
            info = maybe;
            continue;
        }

        if maybe && name == "state" {
            orm_fail(sql, node, "compare a Json_Maybe's state with .MISSING, .NULL or .SET");
            return .[], null, false;
        }

        if info.type != .STRUCT {
            orm_fail(sql, node, tprint("% has no field \"%\"", get_type(info), name));
            return .[], null, false;
        }

        field: *Type_Info_Struct_Member;
        for *(cast(*Type_Info_Struct) info).members if it.name == name field = it;

        if !field {
            orm_fail(sql, node, tprint("% has no field \"%\"", get_type(info), name));
            return .[], null, false;
        }

        array_add(*keys, json_key_of(field));
        info = field.type;
    }

    return keys, info, true;
}

// `("column"->'a'->>'b')`: the last key as text, or as a document when `as_document` is set.
orm_json_access :: (sql: *Orm_Sql, column: string, keys: [] string, as_document: bool) {
    append(*sql.builder, "(");
    append(*sql.builder, quote_identifier(column));

    for keys {
        arrow := ifx it_index == keys.count - 1 && !as_document then "->>" else "->";
        print_to_builder(*sql.builder, "%'%'", arrow, replace(it, "'", "''",, temp));
    }

    append(*sql.builder, ")");
}

// The value type of a `Json_Maybe(T)`, or null for any other type.
json_maybe_value :: (info: *Type_Info) -> *Type_Info {
    if !info || info.type != .STRUCT return null;

    struct_info := cast(*Type_Info_Struct) info;
    if !begins_with(struct_info.name, "Json_Maybe") || struct_info.members.count != 2 return null;
    if struct_info.members[0].name != "state" || struct_info.members[1].name != "value" return null;

    return struct_info.members[1].type;
}

// `it.settings.beta.state == .MISSING` (or `.NULL`, `.SET`, or `!=`) for a `Json_Maybe` field. A
// row whose column is SQL null has none of the three states. Returns false when `binary` is not
// such a comparison.
orm_maybe_state :: (sql: *Orm_Sql, binary: *Code_Binary_Operator) -> bool {
    side, state_node := binary.left, binary.right;
    state := enum_literal_name(state_node);

    if !state {
        side, state_node = binary.right, binary.left;
        state = enum_literal_name(state_node);
    }

    if !state return false;

    path := field_path(side);
    if path.count < 3 || path[path.count - 1] != "state" return false;

    member := member_of(sql.T, path[0]);
    if !member return false;

    keys, info, ok := orm_json_keys(sql, side, member, array_view(path, 0, path.count - 1));
    if !ok return true;

    if !json_maybe_value(info) || !keys.count {
        orm_fail(sql, side, "only a Json_Maybe field inside a json column has a state");
        return true;
    }

    parent := array_view(keys, 0, keys.count - 1);
    key := replace(keys[keys.count - 1], "'", "''",, temp);

    append(*sql.builder, ifx binary.operator_type == .IS_EQUAL then "coalesce(" else "not coalesce(");

    if state == {
        case "MISSING";
            append(*sql.builder, "not (");
            orm_json_access(sql, path[0], parent, true);
            print_to_builder(*sql.builder, "::jsonb ? '%')", key);
        case "NULL";
            orm_json_access(sql, path[0], keys, true);
            append(*sql.builder, "::jsonb = 'null'::jsonb");
        case "SET";
            append(*sql.builder, "jsonb_typeof(");
            orm_json_access(sql, path[0], keys, true);
            append(*sql.builder, "::jsonb) <> 'null'");
        case;
            orm_fail(sql, state_node, tprint("Json_Maybe has no state .%", state));
            return true;
    }

    append(*sql.builder, ", false)");

    return true;
}

// `MISSING` for the enum literal `.MISSING`, else empty.
enum_literal_name :: (node: *Code_Node) -> string {
    if !node || node.kind != .BINARY_OPERATOR return "";

    binary := cast(*Code_Binary_Operator) node;
    if binary.operator_type != #char "." || binary.left || !binary.right || binary.right.kind != .IDENT return "";

    return (cast(*Code_Ident) binary.right).name;
}

// The key a field has in its json document: its `@json(name=...)` rename, else its name.
json_key_of :: (member: *Type_Info_Struct_Member) -> string {
    for member.notes {
        arguments, found := note_arguments(it, "json");
        if !found continue;

        for split(arguments, ",",, temp) {
            part := trim(it);
            if begins_with(part, "name=") return trim(advance(part, 5), "\"");
            if begins_with(part, "name =") return trim(advance(part, 6), " \"");
        }
    }

    return member.name;
}

orm_binary :: (sql: *Orm_Sql, binary: *Code_Binary_Operator) {
    op := binary.operator_type;

    if op == #char "." {
        orm_column(sql, binary);
        return;
    }

    if op == .IS_EQUAL || op == .IS_NOT_EQUAL {
        if orm_maybe_state(sql, binary) return;

        null_side: *Code_Node;
        other: *Code_Node;

        if is_null_code(binary.right) {
            null_side = binary.right;
            other = binary.left;
        } else if is_null_code(binary.left) {
            null_side = binary.left;
            other = binary.right;
        }

        if null_side {
            append(*sql.builder, "(");
            orm_expression(sql, other);
            append(*sql.builder, ifx op == .IS_EQUAL then " is null)" else " is not null)");
            return;
        }
    }

    word := "";

    if op == {
        case .IS_EQUAL; word = "=";
        case .IS_NOT_EQUAL; word = "<>";
        case .LOGICAL_AND; word = "and";
        case .LOGICAL_OR; word = "or";
        case .LESS_OR_EQUAL; word = "<=";
        case .GREATER_OR_EQUAL; word = ">=";
        case #char "<"; word = "<";
        case #char ">"; word = ">";
        case #char "+"; word = "+";
        case #char "-"; word = "-";
        case #char "*"; word = "*";
        case #char "/"; word = "/";
        case #char "%"; word = "%%";
    }

    if !word {
        orm_fail(sql, binary, "the ORM has no SQL for this op");
        return;
    }

    append(*sql.builder, "(");

    // `.admin` has no type of its own: it takes the enum of the column on the other side.
    compares := op == .IS_EQUAL || op == .IS_NOT_EQUAL || op == .LESS_OR_EQUAL || op == .GREATER_OR_EQUAL || op == #char "<" || op == #char ">";

    if compares && enum_literal_name(binary.right) {
        orm_expression(sql, binary.left);
        print_to_builder(*sql.builder, " % ", word);
        orm_enum_hole(sql, binary.left, binary.right);
    } else if compares && enum_literal_name(binary.left) {
        orm_enum_hole(sql, binary.right, binary.left);
        print_to_builder(*sql.builder, " % ", word);
        orm_expression(sql, binary.right);
    } else {
        orm_expression(sql, binary.left);
        print_to_builder(*sql.builder, " % ", word);
        orm_expression(sql, binary.right);
    }

    append(*sql.builder, ")");
}

// The enum literal `literal` as a parameter of the type of `column`: a field of the table, or
// of a to-one relation.
orm_enum_hole :: (sql: *Orm_Sql, column: *Code_Node, literal: *Code_Node) {
    path := field_path(column);
    owner := "";

    if path.count == 1 && sql.T == sql.statement_type {
        owner = "T";
    } else if path.count == 2 && sql.T == sql.statement_type {
        relation, problem := orm_relation(sql.T, path[0]);

        if !problem && relation.kind == .BELONGS_TO {
            index := array_find_index(sql.relation_names, relation.target);

            if index < 0 {
                array_add(*sql.relation_names, relation.target);
                index = sql.relation_names.count - 1;
            }

            owner = tprint("Orm_R%", index);
        }
    }

    if !owner {
        orm_fail(sql, literal, "the ORM cannot tell the type of this enum literal here: name its type (`Role.admin`)");
        return;
    }

    orm_hole(sql, tprint("type_of(%.%).%", owner, path[path.count - 1], enum_literal_name(literal)), in_module = true);
}

orm_call :: (sql: *Orm_Sql, call: *Code_Procedure_Call) {
    name := called_name(call);
    count := call.arguments_unsorted.count;

    if (name == "like" || name == "ilike") && count == 2 {
        append(*sql.builder, "(");
        orm_expression(sql, call_argument(call, 0));
        print_to_builder(*sql.builder, " % ", name);
        orm_expression(sql, call_argument(call, 1));
        append(*sql.builder, ")");

        return;
    }

    if name == "in" && count == 2 {
        append(*sql.builder, "(");
        orm_expression(sql, call_argument(call, 0));
        append(*sql.builder, " = any(");
        orm_expression(sql, call_argument(call, 1));
        append(*sql.builder, "))");

        return;
    }

    if (name == "lower" || name == "upper") && count == 1 {
        print_to_builder(*sql.builder, "%(", name);
        orm_expression(sql, call_argument(call, 0));
        append(*sql.builder, ")");
        return;
    }

    if name == "if_some" && count == 2 {
        orm_if_some(sql, call);
        return;
    }

    if (name == "contains" || name == "has_key" || name == "json_path") && count == 2 {
        orm_json_operator(sql, call, name);
        return;
    }

    if name == "sql" && count == 1 {
        text := call_argument(call, 0);

        if text.kind != .LITERAL || (cast(*Code_Literal) text).value_type != .STRING {
            orm_fail(sql, text, "Pg.sql takes a string literal");
            return;
        }

        append(*sql.builder, "(");
        orm_fragment(sql, call, (cast(*Code_Literal) text)._string);
        append(*sql.builder, ")");

        return;
    }

    orm_fail(sql, call, tprint("the ORM cannot call % in SQL: filters may use Pg.like, Pg.ilike, Pg.in, Pg.lower, Pg.upper, Pg.if_some, Pg.contains, Pg.has_key, Pg.json_path and Pg.sql", ifx name then name else "this"));
}

// `Pg.if_some(team, #code it.team_id == team)`: `($1 or team_id = $2)`, with `team.null` and
// `team.value`.
// `contains`, `has_key` and `json_path` on a jsonb column or a document inside one.
orm_json_operator :: (sql: *Orm_Sql, call: *Code_Procedure_Call, name: string) {
    document := call_argument(call, 0);
    path := field_path(document);

    if !path.count {
        orm_fail(sql, document, tprint("Pg.%'s first argument is a json column, `it.<field>`", name));
        return;
    }

    member := member_of(sql.T, path[0]);
    info := ifx member then member.type else null;

    if info && is_nullable_info(info) info = nullable_value_type(info);

    if info && !is_json_info(info) {
        orm_fail(sql, document, tprint("%.% is not a json column", sql.T, path[0]));
        return;
    }

    append(*sql.builder, "(");
    orm_column_path(sql, document, path, as_document = true);

    value := call_argument(call, 1);

    if name == {
        case "contains";
            append(*sql.builder, " @> ");
            orm_parameter(sql, value, "orm_json_value");
            append(*sql.builder, "::jsonb");

        case "has_key";
            append(*sql.builder, " ? ");
            orm_expression(sql, value);

        case "json_path";
            append(*sql.builder, " @? ");
            orm_expression(sql, value);
            append(*sql.builder, "::jsonpath");
    }

    append(*sql.builder, ")");
}

// The text of a `Pg.sql` fragment or `from_sql`, with its `%` holes filled: `%(it.a.b)` is a
// column, `%name` and `%(expression)` are parameters, `%%` stays a literal percent sign.
orm_fragment :: (sql: *Orm_Sql, node: *Code_Node, text: string) {
    i := 0;

    while i < text.count {
        c := text[i];

        if c != #char "%" {
            append(*sql.builder, c);
            i += 1;
            continue;
        }

        if i + 1 < text.count && text[i + 1] == #char "%" {
            append(*sql.builder, "%%");
            i += 2;
            continue;
        }

        expression: string;

        if i + 1 < text.count && text[i + 1] == #char "(" {
            depth := 0;
            end := i + 1;

            while end < text.count {
                if text[end] == #char "(" depth += 1;
                if text[end] == #char ")" depth -= 1;
                if depth == 0 break;

                end += 1;
            }

            if end >= text.count {
                orm_fail(sql, node, tprint("unclosed \%( in \"%\"", text));
                return;
            }

            expression = trim(slice(text, i + 2, end - i - 2));
            i = end + 1;
        } else {
            end := i + 1;
            while end < text.count && (is_alnum(text[end]) || text[end] == #char "_") end += 1;

            if end == i + 1 {
                orm_fail(sql, node, tprint("a \% in \"%\" is followed by neither a name nor (expression); write \%\% for a percent sign", text));
                return;
            }

            expression = slice(text, i + 1, end - i - 1);
            i = end;
        }

        if expression == "it" || begins_with(expression, "it.") {
            path := split(expression, ".",, temp);

            for path {
                if !it.count {
                    orm_fail(sql, node, tprint("\"%\" is not a field of it", expression));
                    return;
                }
            }

            orm_column_path(sql, node, array_view(path, 1));
        } else {
            orm_hole(sql, expression);
        }

        if sql.problem return;
    }
}

orm_if_some :: (sql: *Orm_Sql, call: *Code_Procedure_Call) {
    value := call_argument(call, 0);
    condition := call_argument(call, 1);

    if !value || value.kind != .IDENT {
        orm_fail(sql, call, "Pg.if_some takes a variable (a Nullable) and a #code condition that names it");
        return;
    }

    while condition && condition.kind == .DIRECTIVE_CODE condition = (cast(*Code_Directive_Code) condition).expression;

    if !condition {
        orm_fail(sql, call, "Pg.if_some needs a #code condition");
        return;
    }

    saved_name := sql.some_name;
    name := (cast(*Code_Ident) value).name;

    sql.some_name = name;

    // The null test reads the Nullable's flag; the condition reads its value.
    append(*sql.builder, "(");
    orm_hole(sql, tprint("%.null", name));
    append(*sql.builder, " or ");
    orm_expression(sql, condition);
    append(*sql.builder, ")");

    sql.some_name = saved_name;
}

// `set = #code it.a = x` or `#code { it.a = x; it.b = y; }`.
orm_set :: (sql: *Orm_Sql, node: *Code_Node) {
    if node.kind == .BLOCK {
        block := cast(*Code_Block) node;
        first := true;

        for block.statements {
            if !first append(*sql.builder, ", ");
            first = false;
            orm_set(sql, it);
        }

        return;
    }

    if node.kind != .BINARY_OPERATOR || (cast(*Code_Binary_Operator) node).operator_type != #char "=" {
        orm_fail(sql, node, "set takes assignments to fields: #code it.field = value");
        return;
    }

    binary := cast(*Code_Binary_Operator) node;
    path := field_path(binary.left);

    if path.count != 1 || !member_of(sql.T, path[0]) {
        orm_fail(sql, binary.left, tprint("set assigns to a field of % (it.field = value)", sql.T));
        return;
    }

    print_to_builder(*sql.builder, "% = ", quote_identifier(path[0]));
    orm_expression(sql, binary.right);
}

// The SQL of a statement, or a problem reported at the node it is about.
orm_build :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string, types: [] Type, relations: [] Type, columns := "") -> Orm_Sql {
    sql: Orm_Sql;
    sql.T = T;
    sql.statement_type = T;
    sql.builder.allocator = temp;
    sql.parameters.allocator = temp;
    sql.wraps.allocator = temp;
    sql.relation_names.allocator = temp;
    sql.joins.allocator = temp;
    sql.parameter_types = types;
    sql.relation_types = relations;

    table, problem := orm_table(T);

    if problem {
        sql.problem = problem;
        return sql;
    }

    sql.outer = ifx from_sql then "orm_source" else table;

    if kind == {
        case .SELECT;
            append(*sql.builder, "select ");

            if columns {
                written := 0;

                for split(columns, ",",, temp) {
                    entry := orm_read_entry(it);
                    if entry.many && !entry.lateral continue; // loaded after the query

                    if written append(*sql.builder, ", ");

                    orm_read_column(*sql, entry);
                    written += 1;
                }
            } else {
                for orm_columns(T) {
                    if it_index append(*sql.builder, ", ");
                    append(*sql.builder, quote_identifier(it.name));
                }
            }

            if from_sql {
                append(*sql.builder, " from (");
                orm_fragment(*sql, orm_root(where), from_sql);
                append(*sql.builder, ") as orm_source");
            } else {
                print_to_builder(*sql.builder, " from %", table);
            }

            for sql.joins print_to_builder(*sql.builder, " %", it);

        case .UPDATE;
            print_to_builder(*sql.builder, "update % set ", table);
            orm_set(*sql, orm_root(set));

        case .DELETE;
            print_to_builder(*sql.builder, "delete from %", table);
    }

    filter := orm_root(where);

    if filter && !(filter.kind == .LITERAL && (cast(*Code_Literal) filter).value_type == .BOOLEAN && (cast(*Code_Literal) filter)._s64 != 0) {
        append(*sql.builder, " where ");
        orm_expression(*sql, filter);
    }

    if kind == .SELECT {
        order := orm_root(order_by);

        if order && !is_null_code(order) {
            append(*sql.builder, " order by ");
            orm_expression(*sql, order);
            if desc append(*sql.builder, " desc");
        }

        if limit >= 0 print_to_builder(*sql.builder, " limit %", limit);
        if offset > 0 print_to_builder(*sql.builder, " offset %", offset);
    }

    return sql;
}

// One selected column of `each`: a column, a json document cut down to the keys the loop reads
// (`jsonb_each` keeps a key holding null apart from a missing one), or a related field.
orm_read_column :: (sql: *Orm_Sql, entry: Orm_Read_Entry) {
    if entry.lateral {
        orm_lateral_columns(sql, entry.name);
        return;
    }

    if entry.relation {
        orm_related_column(sql, null, .[entry.relation, entry.name], false);
        print_to_builder(*sql.builder, " as %", quote_identifier(orm_relation_alias(entry)));
        return;
    }

    column := quote_identifier(entry.name);

    if !entry.pruned {
        append(*sql.builder, column);
        return;
    }

    // A column that cannot be NULL gets an expression the describe step also knows is never NULL.
    nullable := orm_is_nullable(orm_member_type(sql.T, entry.name));

    if !entry.fields.count {
        if nullable print_to_builder(*sql.builder, "case when % is null then null else '{}'::jsonb end as %", column, column);
        else print_to_builder(*sql.builder, "'{}'::jsonb as %", column);

        return;
    }

    document := cast(*Type_Info_Struct) type_info(orm_json_document_type(sql.T, entry.name));

    if nullable print_to_builder(*sql.builder, "case when % is null then null else ", column);

    print_to_builder(*sql.builder, "coalesce((select jsonb_object_agg(orm_k, orm_v) from jsonb_each(%::jsonb) as orm_j(orm_k, orm_v) where orm_k in (", column);

    for field: entry.fields {
        if it_index append(*sql.builder, ", ");

        for *document.members if it.name == field append(*sql.builder, orm_sql_string(json_key_of(it)));
    }

    append(*sql.builder, ")), '{}'::jsonb)");
    if nullable append(*sql.builder, " end");

    print_to_builder(*sql.builder, " as %", column);
}

// `text` as a SQL string literal.
orm_sql_string :: (text: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    append(*builder, "'");

    for text {
        if it == #char "'" append(*builder, "''");
        else append(*builder, it);
    }

    append(*builder, "'");

    return builder_to_string(*builder);
}

// The file that called the ORM. The typed statements the ORM writes sit in this module, but they
// belong to that file's project: its migrations, query cache and `.env`.
orm_project_file :: (call: Code) -> string {
    return node_location(orm_root(call)).fully_pathed_filename;
}

// `, project_file = "<the caller's file>"`, the last argument of a statement the ORM writes.
orm_project_argument :: (call: Code) -> string {
    return tprint(", project_file = %", jai_string(orm_project_file(call)));
}

// The row type of a `select`: `T`, or `Partial(T, fields)`.
orm_select_row :: (fields: string) -> string {
    return ifx fields then "Partial(T, fields)" else "T";
}

// `fields` as `orm_read_columns` writes column lists: names split on commas, trimmed, joined by ",".
orm_field_list :: (T: Type, fields: string, call: Code = NO_ORDER) -> string, ok: bool {
    builder: String_Builder;
    builder.allocator = temp;

    for split(fields, ",",, temp) {
        name := trim(it);

        if !name || !member_of(T, name) || (member_of(T, name).flags & .CONSTANT) {
            message := tprint("% has no column \"%\" to select", T, name);
            root := orm_root(call);

            if root compiler_report(message, node_location(root));
            else compiler_report(message);

            return "", false;
        }

        if it_index append(*builder, ",");

        append(*builder, name);
    }

    return builder_to_string(*builder), true;
}

orm_partial_body :: (T: Type, fields: string) -> string {
    list, ok := orm_field_list(T, fields);
    if !ok return "";

    builder: String_Builder;
    builder.allocator = temp;

    for name: split(list, ",",, temp) {
        print_to_builder(*builder, "%: #run orm_member_type(T, %);", name, jai_string(name));
        for member_of(T, name).notes print_to_builder(*builder, " @%", it);
        append(*builder, "\n");
    }

    return builder_to_string(*builder);
}

// The body of `select`, `update` and `delete`: a constant per parameter for its type, then the
// typed query with the SQL built from those types.
orm_statement_code :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string, fields: string, call: Code) -> string {
    columns: string;

    if fields {
        ok: bool;
        columns, ok = orm_field_list(T, fields, call);
        if !ok return "";
    }

    sql := orm_build(T, kind, where, order_by, set, desc, limit, offset, from_sql, .[], .[], columns);

    if sql.problem {
        location := ifx sql.problem_node then node_location(sql.problem_node) else node_location(orm_root(where));
        compiler_report(sql.problem, location);
        return "";
    }

    builder: String_Builder;
    builder.allocator = temp;

    orm_prelude(*builder, *sql);

    kind_text := tprint(".%", kind);
    sql_text := tprint("#run orm_sql_text(T, %, where, %, %, %, %, %, %, Orm_Types, Orm_Relations, %)", kind_text, ifx kind == .SELECT then "order_by" else "where", ifx kind == .UPDATE then "set" else "where", desc, limit, offset, jai_string(from_sql), jai_string(columns));

    if kind == .SELECT print_to_builder(*builder, "orm_result, orm_error := query_as(%, db, %", orm_select_row(fields), sql_text);
    else print_to_builder(*builder, "orm_result, orm_error := execute(db, %", sql_text);

    for sql.parameters print_to_builder(*builder, ", %", orm_argument(*sql, it_index));

    append(*builder, orm_project_argument(call));
    append(*builder, ");\nreturn orm_result, orm_error;\n");

    return builder_to_string(*builder);
}

// Declares each parameter's type (`Orm_P<i>`, typed where the call was written) and each related
// struct (`Orm_R<i>`), and the arrays of both that the second pass of `orm_sql_text` takes.
orm_prelude :: (builder: *String_Builder, sql: *Orm_Sql) {
    for sql.parameters print_to_builder(builder, "Orm_P% :: type_of(%);\n", it_index, orm_argument(sql, it_index));

    append(builder, "Orm_Types :: Type.[");
    for sql.parameters print_to_builder(builder, "%Orm_P%", ifx it_index then ", " else "", it_index);

    append(builder, "];\n");

    for sql.relation_names print_to_builder(builder, "Orm_R% :: %;\n", it_index, orm_insert_text(it));

    append(builder, "Orm_Relations :: Type.[");
    for sql.relation_names print_to_builder(builder, "%Orm_R%", ifx it_index then ", " else "", it_index);

    append(builder, "];\n");
}

// A parameter's Jai text, inserted where the call was written.
orm_insert_text :: (text: string) -> string {
    return tprint("(#insert,scope(call) %)", jai_string(text));
}

// Parameter `index` as an argument: its text where the filter was written, through its wrap.
orm_argument :: (sql: *Orm_Sql, index: s64) -> string {
    text := sql.parameters[index];
    wrap := sql.wraps[index];

    return ifx wrap then tprint("%(%)", wrap, text) else text;
}

orm_sql_text :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string, types: [] Type, relations: [] Type, columns := "") -> string {
    sql := orm_build(T, kind, where, order_by, set, desc, limit, offset, from_sql, types, relations, columns);

    // Problems only the related structs reveal (an unknown field of `it.team`).
    if sql.problem {
        location := ifx sql.problem_node then node_location(sql.problem_node) else node_location(orm_root(where));
        compiler_report(sql.problem, location);
        return "";
    }

    return builder_to_string(*sql.builder);
}

// The body of `each`'s loop. `columns` (from `orm_read_columns`) names what the body reads; the
// rows are read into `Orm_Partial` and copied into `it`, a `T` (or an `Orm_Item` that `using`s one
// and adds the related structs the body reads).
orm_each_code :: (T: Type, where: Code, order_by: Code, desc: bool, limit: s64, offset: s64, from_sql: string, strategy: Orm_Strategy, body: Code, call: Code) -> string {
    columns := orm_read_columns(T, body);
    if strategy == .LATERAL columns = replace(columns, "[]", "[*]",, temp);

    sql := orm_build(T, .SELECT, where, order_by, where, desc, limit, offset, from_sql, .[], .[], columns);

    if sql.problem {
        location := ifx sql.problem_node then node_location(sql.problem_node) else node_location(orm_root(where));
        compiler_report(sql.problem, location);
        return "";
    }

    builder: String_Builder;
    builder.allocator = temp;

    orm_prelude(*builder, *sql);

    entries := ifx columns then split(columns, ",",, temp) else .[];
    copies: String_Builder;
    copies.allocator = temp;
    relations: [..] string;
    relations.allocator = temp;
    many: [..] int; // the entries of has-many relations
    many.allocator = temp;
    preload: [..] int; // those loaded by a second query
    preload.allocator = temp;

    // Declarations first: each pruned document's struct and each related field's type.
    for text, index: entries {
        entry := orm_read_entry(text);

        if entry.many {
            relation := orm_relation(T, entry.name);
            print_to_builder(*builder, "Orm_M% :: %;\n", index, orm_insert_text(relation.target));
            array_add(*many, index);
            if !entry.lateral array_add(*preload, index);
        } else if entry.relation {
            relation_index := array_find_index(sql.relation_names, orm_relation_target_name(T, entry.relation));
            print_to_builder(*builder, "Orm_F% :: #run orm_member_type(Orm_R%, %);\n", index, relation_index, jai_string(entry.name));
            array_add_if_unique(*relations, entry.relation);
        } else if entry.pruned {
            print_to_builder(*builder, "Orm_D% :: #run orm_json_document_type(T, %);\n", index, jai_string(entry.name));
            print_to_builder(*builder, "Orm_Doc% :: struct {\n", index);

            for entry.fields {
                print_to_builder(*builder, "    %1: type_of(Orm_D%2.%1) = #run orm_initial_value(Orm_D%2, %3, type_of(Orm_D%2.%1));", it, index, jai_string(it));
                for orm_json_document_notes(T, entry.name, it) print_to_builder(*builder, " @%", it);
                append(*builder, "\n");
            }

            append(*builder, "}\n");
        }
    }

    if columns {
        append(*builder, "Orm_Partial :: struct {\n");

        for entries {
            entry := orm_read_entry(it);

            if entry.lateral {
                print_to_builder(*builder, "    #insert #run orm_lateral_fields(Orm_M%1, %2, \"Orm_M%1\");\n", it_index, jai_string(entry.name));
                continue;
            }

            if entry.many continue;

            if entry.relation {
                print_to_builder(*builder, "    #if orm_is_nullable(Orm_F%) %: Orm_F%; else %: Nullable(Orm_F%);\n", it_index, orm_relation_alias(entry), it_index, orm_relation_alias(entry), it_index);
            } else if entry.pruned {
                nullable := orm_is_nullable(orm_member_type(T, entry.name));
                print_to_builder(*builder, "    %: %Json(Orm_Doc%)%;\n", entry.name, ifx nullable then "Nullable(" else "", it_index, ifx nullable then ")" else "");
            } else {
                print_to_builder(*builder, "    %: #run orm_member_type(T, %);\n", entry.name, jai_string(entry.name));
            }
        }

        append(*builder, "}\n");
    }

    for relation: relations {
        print_to_builder(*builder, "Orm_Related_% :: struct {\n", relation);

        for entries {
            entry := orm_read_entry(it);
            if entry.relation == relation print_to_builder(*builder, "    %: Orm_F%;\n", entry.name, it_index);
        }

        append(*builder, "}\n");
    }

    if relations || many {
        append(*builder, "Orm_Item :: struct {\n    using orm_row: T;\n");
        for relations print_to_builder(*builder, "    %: Orm_Related_%;\n", it, it);
        for many print_to_builder(*builder, "    %: [] Orm_M%;\n", orm_read_entry(entries[it]).name, it);
        append(*builder, "}\n");
    }

    for entries {
        entry := orm_read_entry(it);
        alias := orm_relation_alias(entry);

        if entry.lateral {
            print_to_builder(*copies, "        #insert #run orm_lateral_copies(Orm_M%1, %2, \"Orm_M%1\");\n", it_index, jai_string(entry.name));
            continue;
        }

        if entry.many continue;

        if entry.relation {
            print_to_builder(*copies, "        #if orm_is_nullable(Orm_F%) it.%.% = orm_partial.%; else it.%.% = orm_partial.%.value;\n", it_index, entry.relation, entry.name, alias, entry.relation, entry.name, alias);
        } else if entry.pruned {
            document := entry.name;

            if orm_is_nullable(orm_member_type(T, entry.name)) {
                print_to_builder(*copies, "        it.%.null = orm_partial.%.null;\n", entry.name, entry.name);
                document = tprint("%.value", entry.name);
            }

            for entry.fields print_to_builder(*copies, "        it.%.value.% = orm_partial.%.value.%;\n", document, it, document, it);
        } else {
            print_to_builder(*copies, "        it.% = orm_partial.%;\n", entry.name, entry.name);
        }
    }

    row_type := ifx columns then "Orm_Partial" else "T";

    print_to_builder(*builder, "orm_rows, orm_error := query_as(%, q.db, #run orm_sql_text(T, .SELECT, where, order_by, where, %, %, %, %, Orm_Types, Orm_Relations, %)", row_type, desc, limit, offset, jai_string(from_sql), jai_string(columns));
    for sql.parameters print_to_builder(*builder, ", %", orm_argument(*sql, it_index));

    append(*builder, orm_project_argument(call));
    append(*builder, ");\n");

    item_type := ifx relations || many then "Orm_Item" else "T";

    // Reads row `it_index` into `it`, or breaks out of the enclosing loop.
    read: String_Builder;
    read.allocator = temp;

    print_to_builder(*read, "        `it: %;\n", item_type);

    if columns {
        append(*read, "        orm_partial: Orm_Partial;\n        orm_read := read_row(*orm_rows, it_index, *orm_partial);\n");
    } else {
        append(*read, "        orm_read := read_row(*orm_rows, it_index, *it);\n");
    }

    append(*read, "        if orm_read.kind != .NONE {\n            if q.error q.error.* = orm_read;\n            orm_failed = true;\n            break;\n        }\n");

    if columns append(*read, builder_to_string(*copies));

    append(*builder, #string JAI
if orm_error.kind != .NONE {
    if q.error q.error.* = orm_error;
} else {
    defer free_rows(*orm_rows);
    orm_failed := false;
JAI);

    if preload {
        // Every row first, so each has-many relation loads in one query for all of them.
        append(*builder, "    orm_items := NewArray(orm_rows.result.row_count, Orm_Item);\n    defer array_free(orm_items);\n");
        append(*builder, "    for it_index: 0..orm_rows.result.row_count - 1 {\n");
        append(*builder, replace(builder_to_string(*read), "`", "",, temp));
        append(*builder, "        orm_items[it_index] = it;\n    }\n");
        append(*builder, "    orm_parents := NewArray(orm_items.count, T,, temp);\n    for orm_items orm_parents[it_index] = it.orm_row;\n");

        for preload {
            name := orm_read_entry(entries[it]).name;
            print_to_builder(*builder, "    orm_many_%1: Loaded_Many(Orm_M%2);\n    defer free_loaded(*orm_many_%1);\n", it, it);
            print_to_builder(*builder, "    if !orm_failed {\n        orm_load_error: Pg_Error;\n        orm_many_%1, orm_load_error = orm_load_many_as(q.db, orm_parents, %2, Orm_M%1, %3);\n", it, jai_string(name), jai_string(orm_project_file(call)));
            append(*builder, "        if orm_load_error.kind != .NONE {\n            if q.error q.error.* = orm_load_error;\n            orm_failed = true;\n        }\n    }\n");
        }

        append(*builder, "    if !orm_failed for `it_index: 0..orm_items.count - 1 {\n        `it := orm_items[it_index];\n");
        for preload print_to_builder(*builder, "        it.% = orm_many_%.items[it_index];\n", orm_read_entry(entries[it]).name, it);
    } else {
        append(*builder, "    for `it_index: 0..orm_rows.result.row_count - 1 {\n");
        append(*builder, builder_to_string(*read));
    }

    append(*builder, #string JAI

        #insert body;
    }
}
JAI);

    return builder_to_string(*builder);
}

// One entry of `orm_read_columns`' list: `column`, `column:field|field` (a json column of which
// only those top-level fields are read) or `relation.field` (a field of a to-one relation).
Orm_Read_Entry :: struct {
    name: string; // the column, or the related struct's field
    relation: string;
    pruned: bool;
    many: bool; // `relation[]`: a has-many relation the body reads whole
    lateral: bool; // `relation[*]`: the same, read in the statement through a lateral join
    fields: [] string;
}

orm_read_entry :: (text: string) -> Orm_Read_Entry {
    entry: Orm_Read_Entry;
    dot := find_index_from_left(text, #char ".");
    colon := find_index_from_left(text, #char ":");

    if ends_with(text, "[]") {
        entry.name = slice(text, 0, text.count - 2);
        entry.many = true;
    } else if ends_with(text, "[*]") {
        entry.name = slice(text, 0, text.count - 3);
        entry.many = true;
        entry.lateral = true;
    } else if dot >= 0 {
        entry.relation = slice(text, 0, dot);
        entry.name = advance(text, dot + 1);
    } else if colon >= 0 {
        entry.name = slice(text, 0, colon);
        entry.pruned = true;

        rest := advance(text, colon + 1);
        if rest entry.fields = split(rest, "|",, temp);
    } else {
        entry.name = text;
    }

    return entry;
}

// A has-many relation read in the statement: a lateral join gathers each column of the related
// rows into an array (`array_agg` keeps the columns' orders the same), and the select list reads
// the arrays as `orm__<relation>__<column>`. No related rows give empty arrays.
orm_lateral_columns :: (sql: *Orm_Sql, name: string) {
    relation, problem := orm_relation(sql.T, name);

    if problem || relation.kind != .HAS_MANY {
        orm_fail(sql, null, ifx problem then problem else tprint("%.% is not a has-many relation", sql.T, name));
        return;
    }

    index := array_find_index(sql.relation_names, relation.target);

    if index < 0 {
        array_add(*sql.relation_names, relation.target);
        index = sql.relation_names.count - 1;
    }

    // The first pass does not know the related struct yet; the text it writes is not used.
    if index >= sql.relation_types.count {
        append(*sql.builder, "null");
        return;
    }

    target := sql.relation_types[index];
    table, table_problem := orm_table(target);

    if table_problem {
        orm_fail(sql, null, table_problem);
        return;
    }

    alias := tprint("orm_m%", sql.joins.count);
    join: String_Builder;
    join.allocator = temp;

    append(*join, "left join lateral (select ");

    for orm_columns(target) {
        column := quote_identifier(it.name);

        if it_index {
            append(*sql.builder, ", ");
            append(*join, ", ");
        }

        // Named apart from the table's own columns, which the statement reads unqualified. `!`:
        // the describe step cannot see through the join that coalesce never gives null.
        joined := quote_identifier(tprint("orm__%__%", name, it.name));

        print_to_builder(*sql.builder, "%.% as %", alias, joined, quote_identifier(tprint("orm__%__%!", name, it.name)));
        print_to_builder(*join, "coalesce(array_agg(%_rows.%), '{}') as %", alias, column, joined);
    }

    print_to_builder(*join, " from % as %_rows where ", table, alias);

    for relation.target_columns {
        if it_index append(*join, " and ");
        print_to_builder(*join, "%_rows.% = %.%", alias, quote_identifier(it), sql.outer, quote_identifier(relation.own_columns[it_index]));
    }

    print_to_builder(*join, ") as % on true", alias);
    array_add(*sql.joins, builder_to_string(*join));
}

// The fields of `Orm_Partial` holding a lateral has-many relation's arrays, one per column.
orm_lateral_fields :: ($Target: Type, name: string, type_name: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;

    for orm_columns(Target) print_to_builder(*builder, "orm__%1__%2: [] type_of(%3.%2);\n", name, it.name, type_name);

    return builder_to_string(*builder);
}

// Rebuilds a lateral has-many relation's rows from its arrays, into `it.<name>`; they are freed
// when the loop moves on.
orm_lateral_copies :: ($Target: Type, name: string, type_name: string) -> string {
    builder: String_Builder;
    builder.allocator = temp;
    columns := orm_columns(Target);

    if !columns.count return "";

    print_to_builder(*builder, "it.%1 = NewArray(orm_partial.orm__%1__%2.count, %3);\n", name, columns[0].name, type_name);
    print_to_builder(*builder, "defer array_free(it.%);\n", name);
    print_to_builder(*builder, "for * it.% {\n", name);
    for columns print_to_builder(*builder, "    it.%2 = orm_partial.orm__%1__%2[it_index];\n", name, it.name);

    append(*builder, "}\n");

    return builder_to_string(*builder);
}

orm_relation_alias :: (entry: Orm_Read_Entry) -> string {
    return tprint("orm__%__%", entry.relation, entry.name);
}

orm_relation_target_name :: (T: Type, name: string) -> string {
    relation := orm_relation(T, name);
    return relation.target;
}

// What a loop body reads through `it`, as a comma-separated list of `Orm_Read_Entry` texts: the
// columns (always the primary key) in the struct's order, then the related fields. A json column
// whose document the body only reads top-level fields of (`it.settings.value.value.theme`) keeps
// just those keys. Every column is listed when the body uses `it` whole; the list is empty when
// that leaves nothing to prune or join.
orm_read_columns :: (T: Type, body: Code) -> string {
    _, nodes := compiler_get_nodes(body);

    // A chain `it.a.b.c` is nested `.` operators; the outermost one is not the left side of another.
    inner: [..] *Code_Node;
    inner.allocator = temp;

    for nodes {
        if it.kind != .BINARY_OPERATOR continue;

        binary := cast(*Code_Binary_Operator) it;
        if binary.operator_type == #char "." array_add(*inner, binary.left);
    }

    Column_Read :: struct {
        name: string;
        whole: bool;
        fields: [..] string;
    }

    uses := 0;
    chains := 0;
    reads: [..] Column_Read;
    reads.allocator = temp;
    related: [..] string;
    related.allocator = temp;

    for nodes {
        if is_ident(it, "it") uses += 1;
        if array_find(inner, it) continue;

        path := orm_it_path(it);
        if !path.count continue;

        chains += 1;

        member := member_of(T, path[0]);

        if !member {
            relation, problem := orm_relation(T, path[0]);
            if !problem && relation.kind == .BELONGS_TO && path.count >= 2 {
                array_add_if_unique(*related, tprint("%.%", path[0], path[1]));
            }

            // The loop loads the rows after the query, from the parents' key columns.
            if !problem && relation.kind == .HAS_MANY {
                array_add_if_unique(*related, tprint("%[]", path[0]));

                for key: relation.own_columns {
                    found := false;
                    for reads if it.name == key found = true;
                    if found continue;

                    read := array_add(*reads);
                    read.name = key;
                    read.whole = true;
                    read.fields.allocator = temp;
                }
            }

            continue;
        }

        read: *Column_Read;
        for *reads if it.name == path[0] read = it;

        if !read {
            read = array_add(*reads);
            read.name = path[0];
            read.fields.allocator = temp;
        }

        field := orm_document_field(member, path);

        if field == "" read.whole = true;
        else if field != "null" array_add_if_unique(*read.fields, field);
    }

    whole := uses > chains;

    builder: String_Builder;
    builder.allocator = temp;
    count := 0;
    pruned := false;
    columns := orm_columns(T);

    for column: columns {
        found: *Column_Read;
        for *reads if it.name == column.name found = it;

        if !whole && !found && !has_note(column, "pg_primary_key") continue;

        if count append(*builder, ",");

        append(*builder, column.name);
        count += 1;

        if whole || !found || found.whole continue;

        // Read only for its nullness or some fields of its document.
        pruned = true;
        append(*builder, ":");
        for found.fields print_to_builder(*builder, "%1%2", ifx it_index then "|" else "", it);
    }

    if !pruned && !related.count && count == columns.count return "";

    for related print_to_builder(*builder, ",%", it);

    return builder_to_string(*builder);
}

// `["a", "b", "c"]` for `it.a.b.c`; empty when `node` is not such a chain.
orm_it_path :: (node: *Code_Node) -> [] string {
    names: [..] string;
    names.allocator = temp;

    while node && node.kind == .BINARY_OPERATOR {
        binary := cast(*Code_Binary_Operator) node;
        if binary.operator_type != #char "." || !binary.right || binary.right.kind != .IDENT return .[];

        array_insert_at(*names, (cast(*Code_Ident) binary.right).name, 0);
        node = binary.left;
    }

    if !is_ident(node, "it") return .[];

    return names;
}

// For a read `it.<column>...` of a json column holding a struct: the document's top-level field
// it reads, "null" when it only reads the column's nullness, or "" when it needs the whole value.
orm_document_field :: (member: *Type_Info_Struct_Member, path: [] string) -> string {
    info := member.type;
    at := 1;

    if is_nullable_info(info) {
        if path.count == at return "";
        if path[at] == "null" return "null";
        if path[at] != "value" return "";

        info = nullable_value_type(info);
        at += 1;
    }

    if !is_json_info(info) || path.count <= at || path[at] != "value" return "";

    document := nullable_value_type(info);

    at += 1;

    if document.type != .STRUCT || path.count <= at return "";

    for (cast(*Type_Info_Struct) document).members if it.name == path[at] && !(it.flags & .CONSTANT) return it.name;

    return "";
}

orm_member_type :: (T: Type, name: string) -> Type {
    member := member_of(T, name);

    if !member {
        compiler_report(tprint("% has no field \"%\"", T, name));
        return void;
    }

    return get_type(member.type);
}

// The struct a json column holds: `Settings` for `Nullable(Json(Settings))`.
orm_json_document_type :: (T: Type, column: string) -> Type {
    info := member_of(T, column).type;
    if is_nullable_info(info) info = nullable_value_type(info);

    return get_type(nullable_value_type(info));
}

// The notes of a document field, so the pruned struct decodes it the same way.
orm_json_document_notes :: (T: Type, column: string, field: string) -> [] string {
    document := cast(*Type_Info_Struct) type_info(orm_json_document_type(T, column));

    for document.members if it.name == field return it.notes;

    return .[];
}

// A field's initial value in a fresh `Document`, for a field `@json(default)` leaves unset.
orm_initial_value :: ($Document: Type, field: string, $Field: Type) -> Field {
    document: Document;
    value: Field;

    for (cast(*Type_Info_Struct) type_info(Document)).members {
        if it.name == field value = (cast(*Field) ((cast(*u8) *document) + it.offset_in_bytes)).*;
    }

    return value;
}

orm_copy_code :: (T: Type) -> string {
    table, problem := orm_table(T);

    if problem {
        compiler_report(problem);
        return "";
    }

    names: String_Builder;
    names.allocator = temp;
    members: String_Builder;
    members.allocator = temp;
    count := 0;

    for orm_columns(T) {
        if has_note(it, "pg_default") || has_note(it, "pg_generated") continue;

        if count {
            append(*names, ", ");
            append(*members, ", ");
        }

        append(*names, quote_identifier(it.name));
        append(*members, jai_string(it.name));
        count += 1;
    }

    if !count {
        compiler_report(tprint("% has no field to copy: every one is @pg_default or @pg_generated", T));
        return "";
    }

    statement := tprint("copy % (%) from stdin (format binary)", table, builder_to_string(*names,, temp));

    return tprint("stored, error := copy_in(db, %, values, .[%]);\nreturn stored, error;\n", jai_string(statement), builder_to_string(*members,, temp));
}

orm_insert_code :: (T: Type, call: Code) -> string {
    table, problem := orm_table(T);

    if problem {
        compiler_report(problem);
        return "";
    }

    columns := orm_columns(T);
    names: String_Builder;
    names.allocator = temp;
    holes: String_Builder;
    holes.allocator = temp;
    values: String_Builder;
    values.allocator = temp;
    all: String_Builder;
    all.allocator = temp;

    inserted := 0;

    for columns {
        if it_index append(*all, ", ");

        append(*all, quote_identifier(it.name));

        if has_note(it, "pg_default") || has_note(it, "pg_generated") continue;

        if inserted {
            append(*names, ", ");
            append(*holes, ", ");
        }

        append(*names, quote_identifier(it.name));
        append(*holes, "%");

        cast_to := parameter_cast(get_type(it.type));
        if cast_to print_to_builder(*holes, "::%", cast_to);

        print_to_builder(*values, ", value.%", it.name);
        inserted += 1;
    }

    statement := ifx inserted then tprint("insert into % (%) values (%) returning %", table, builder_to_string(*names,, temp), builder_to_string(*holes,, temp), builder_to_string(*all,, temp)) else tprint("insert into % default values returning %", table, builder_to_string(*all,, temp));

    builder: String_Builder;
    builder.allocator = temp;

    print_to_builder(*builder, "rows, error := query_as(T, db, %1%2%3);\n", jai_string(statement), builder_to_string(*values,, temp), orm_project_argument(call));
    append(*builder, "row: T;\n");
    append(*builder, "if error.kind != .NONE return row, error;\n");
    append(*builder, "detached := Rows(T).{ result = detach_to_temp(*rows.result), map = rows.map };\n");
    append(*builder, "if detached.result.row_count != 1 return row, make_error(.USAGE, \"the insert returned % rows\", detached.result.row_count);\n");
    append(*builder, "error = read_row(*detached, 0, *row);\n");
    append(*builder, "return row, error;\n");

    return builder_to_string(*builder);
}
