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
// `from_sql = "select * from users_visible_to(%viewer)"`.
select :: (db: $D, $T: Type, $where: Code = EVERY_ROW, $order_by: Code = NO_ORDER, $desc := false, $limit := -1, $offset := 0, $from_sql := "", $call := #caller_code) -> Rows(T), Pg_Error #expand {
    #insert #run orm_statement_code(T, .SELECT, where, order_by, where, desc, limit, offset, from_sql);
}

// Sets fields of the rows that pass `where`: `set = #code it.score = score + 1` (or a block of such
// assignments). Returns the number of rows changed.
update :: (db: $D, $T: Type, $set: Code, $where: Code, $call := #caller_code) -> s64, Pg_Error #expand {
    #insert #run orm_statement_code(T, .UPDATE, where, where, set, false, -1, 0, "");
}

// Deletes the rows that pass `where` and returns how many there were. `where` is required: pass
// `#code true` to delete every row.
delete :: (db: $D, $T: Type, $where: Code, $call := #caller_code) -> s64, Pg_Error #expand {
    #insert #run orm_statement_code(T, .DELETE, where, where, where, false, -1, 0, "");
}

// Inserts `value` and returns the row the table now holds (with its defaults and generated
// columns). Fields with a database default (`@pg_default`) or a generated value
// (`@pg_generated`) are left to the database.
insert :: (db: $D, value: $T) -> T, Pg_Error #expand {
    #insert #run orm_insert_code(T);
}

// Inserts each value in turn (one statement each), stopping at the first error. Returns the rows
// inserted so far, from context.allocator.
insert_many :: (db: $D, values: [] $T) -> [] T, Pg_Error {
    rows: [..] T;

    for values {
        row, error := insert(db, it);
        if error.kind != .NONE return rows, error;

        array_add(*rows, row);
    }

    return rows, .{};
}

// The rows `parents` have through the has-many relation `relation` (a `@pg_has_many` note of
// `T`), in one query: `items[i]` belongs to `parents[i]`. Free it with `free_loaded`; the items
// read from its result.
//
//     posts, error := Pg.load_many(db, users, "posts");
//     for users print("% wrote % posts\n", it.email, posts.items[it_index].count);
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

load_many :: (db: $D, parents: [] $T, $relation: string, $call := #caller_code) -> Loaded_Many(#insert,scope(call) #run orm_relation_target(T, relation, .HAS_MANY)), Pg_Error #expand {
    #insert #run orm_load_code(T, relation, .HAS_MANY);
}

load_one :: (db: $D, parents: [] $T, $relation: string, $call := #caller_code) -> Loaded_One(#insert,scope(call) #run orm_relation_target(T, relation, .BELONGS_TO)), Pg_Error #expand {
    #insert #run orm_load_code(T, relation, .BELONGS_TO);
}

free_loaded :: (loaded: *Loaded_Many($Item)) {
    array_free(loaded.items);
    array_free(loaded.rows);
    free_result(*loaded.result);
    loaded.* = .{};
}

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
like :: (value: string, pattern: string) -> bool {
    return false;
}

// `ilike`: case-insensitive `like`.
ilike :: (value: string, pattern: string) -> bool {
    return false;
}

// `= any(values)`: `Pg.in(it.id, ids)`.
in :: (value: $T, values: [] T) -> bool {
    return false;
}

// `lower(value)`.
lower :: (value: string) -> string {
    return "";
}

// `upper(value)`.
upper :: (value: string) -> string {
    return "";
}

// `(value is null or condition)`: an optional filter, `Pg.if_some(team, #code it.team_id == team)`
// with `team: Nullable(s64)`, where `team` stands for its value inside the condition.
if_some :: (value: $T, condition: Code) -> bool {
    return false;
}

// `?`: the jsonb document has the top-level key: `Pg.has_key(it.settings, "beta")`.
has_key :: (document: $T, key: string) -> bool {
    return false;
}

// `@?`: the jsonpath matches: `Pg.json_path(it.settings, "$.tags[*] ? (@ == \"x\")")`.
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
    builder: String_Builder;
    parameters: [..] string;
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

    if !problem && relation.own_columns.count != 1 problem = tprint("%.% joins on several columns, which loading does not handle yet", T, name);

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

// `select <every column of Target> from <its table> where "<column>" = any($1)`.
orm_load_sql :: (Target: Type, column: string, Key: Type) -> string {
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

    cast_to := parameter_cast(Key);

    print_to_builder(*builder, " from % where % = any(\% %)", table, quote_identifier(column), ifx cast_to then tprint("::%[]", cast_to) else "");

    return builder_to_string(*builder);
}

// The body of `load_many` and `load_one`: collect the keys, run one query, and hand each row to
// the parents whose key it carries.
orm_load_code :: (T: Type, name: string, kind: Orm_Relation_Kind) -> string {
    relation, problem := orm_relation(T, name);
    if problem || relation.kind != kind || relation.own_columns.count != 1 return "";

    own := relation.own_columns[0];
    target := relation.target_columns[0];
    own_member := member_of(T, own);

    if !own_member {
        compiler_report(tprint("% has no field \"%\" for its relation \"%\"", T, own, name));
        return "";
    }

    own_nullable := is_nullable_info(own_member.type);
    own_value := ifx own_nullable then tprint("it.%.value", own) else tprint("it.%", own);
    own_skip := ifx own_nullable then tprint("if it.%.null continue;\n", own) else "";

    builder: String_Builder;
    builder.allocator = temp;

    loaded_type := ifx kind == .HAS_MANY then "Loaded_Many" else "Loaded_One";

    print_to_builder(*builder, #string DONE
Orm_Target :: %3;
Orm_Key :: #run orm_value_type(type_of(parents[0].%1));
Orm_Target_Nullable :: #run orm_is_nullable(type_of(Orm_Target.{}.%2));
orm_loaded: %4(Orm_Target);
orm_keys: [..] Orm_Key;
orm_keys.allocator = temp;
for parents {
    %5array_add(*orm_keys, %6);
}
orm_rows, orm_error := query_as(Orm_Target, db, #run orm_load_sql(Orm_Target, "%2", Orm_Key), cast([] Orm_Key) orm_keys);
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
DONE, own, target, orm_insert_text(relation.target), loaded_type, own_skip, own_value);

    if kind == .HAS_MANY {
        print_to_builder(*builder, #string DONE
// Parent key to its first index; parents sharing a key share their items.
orm_parent: Orm_Hash.Table(Orm_Key, s64);
orm_parent.allocator = temp;
for parents {
    %2if !Orm_Hash.table_contains(*orm_parent, %3) Orm_Hash.table_add(*orm_parent, %3, it_index);
}
orm_counts := NewArray(parents.count, s64,, temp);
orm_child_key :: (row: Orm_Target) -> Orm_Key, bool {
    #if Orm_Target_Nullable return row.%1.value, !row.%1.null;
    else return row.%1, true;
}
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
    %2index := Orm_Hash.table_find_pointer(*orm_parent, %3);
    orm_loaded.items[it_index] = array_view(orm_grouped, orm_starts[index.*], orm_counts[index.*]);
}
return orm_loaded, .{};
DONE, target, own_skip, own_value);
    } else {
        print_to_builder(*builder, #string DONE
orm_found_at: Orm_Hash.Table(Orm_Key, s64);
orm_found_at.allocator = temp;
for orm_found {
    #if Orm_Target_Nullable {
        if it.%1.null continue;
        Orm_Hash.table_set(*orm_found_at, it.%1.value, it_index);
    } else {
        Orm_Hash.table_set(*orm_found_at, it.%1, it_index);
    }
}
orm_loaded.items = NewArray(parents.count, Nullable(Orm_Target));
for parents {
    orm_loaded.items[it_index].null = true;
    %2index := Orm_Hash.table_find_pointer(*orm_found_at, %3);
    if index orm_loaded.items[it_index] = .{ value = orm_found[index.*] };
}
return orm_loaded, .{};
DONE, target, own_skip, own_value);
    }

    return builder_to_string(*builder);
}

// A value sent as a json document (for `contains`).
orm_json_value :: (value: $V) -> Json(V) {
    return .{ value = value };
}

orm_hole :: (sql: *Orm_Sql, text: string, wrap := "") {
    index := sql.parameters.count;

    array_add(*sql.parameters, text);
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
    info := member.type;
    if is_nullable_info(info) info = nullable_value_type(info);

    if !is_json_info(info) {
        orm_fail(sql, node, tprint("%.% is not a json column, so it has no fields in SQL", sql.T, path[0]));
        return;
    }

    info = nullable_value_type(info);

    keys: [..] string;
    keys.allocator = temp;

    for 1..path.count - 1 {
        name := path[it];

        if info.type != .STRUCT {
            orm_fail(sql, node, tprint("% has no field \"%\"", get_type(info), name));
            return;
        }

        field: *Type_Info_Struct_Member;
        for *(cast(*Type_Info_Struct) info).members if it.name == name field = it;

        if !field {
            orm_fail(sql, node, tprint("% has no field \"%\"", get_type(info), name));
            return;
        }

        array_add(*keys, json_key_of(field));
        info = field.type;
    }

    append(*sql.builder, "(");
    append(*sql.builder, quote_identifier(path[0]));

    for keys {
        arrow := ifx it_index == keys.count - 1 && !as_document then "->>" else "->";
        print_to_builder(*sql.builder, "%'%'", arrow, replace(it, "'", "''",, temp));
    }

    append(*sql.builder, ")");

    if as_document return;

    leaf := info;
    if is_nullable_info(leaf) leaf = nullable_value_type(leaf);

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
    orm_expression(sql, binary.left);
    print_to_builder(*sql.builder, " % ", word);
    orm_expression(sql, binary.right);
    append(*sql.builder, ")");
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
orm_build :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string, types: [] Type, relations: [] Type) -> Orm_Sql {
    sql: Orm_Sql;
    sql.T = T;
    sql.builder.allocator = temp;
    sql.parameters.allocator = temp;
    sql.wraps.allocator = temp;
    sql.relation_names.allocator = temp;
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

            for orm_columns(T) {
                if it_index append(*sql.builder, ", ");
                append(*sql.builder, quote_identifier(it.name));
            }

            if from_sql {
                append(*sql.builder, " from (");
                orm_fragment(*sql, orm_root(where), from_sql);
                append(*sql.builder, ") as orm_source");
            } else {
                print_to_builder(*sql.builder, " from %", table);
            }

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

// The body of `select`, `update` and `delete`: a constant per parameter for its type, then the
// typed query with the SQL built from those types.
orm_statement_code :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string) -> string {
    sql := orm_build(T, kind, where, order_by, set, desc, limit, offset, from_sql, .[], .[]);

    if sql.problem {
        location := ifx sql.problem_node then node_location(sql.problem_node) else node_location(orm_root(where));
        compiler_report(sql.problem, location);
        return "";
    }

    builder: String_Builder;
    builder.allocator = temp;

    for sql.parameters print_to_builder(*builder, "Orm_P% :: type_of(%);\n", it_index, orm_argument(*sql, it_index));

    append(*builder, "Orm_Types :: Type.[");
    for sql.parameters print_to_builder(*builder, "%Orm_P%", ifx it_index then ", " else "", it_index);

    append(*builder, "];\n");

    // The related structs, named where the call was written.
    for sql.relation_names print_to_builder(*builder, "Orm_R% :: %;\n", it_index, orm_insert_text(it));

    append(*builder, "Orm_Relations :: Type.[");
    for sql.relation_names print_to_builder(*builder, "%Orm_R%", ifx it_index then ", " else "", it_index);

    append(*builder, "];\n");

    kind_text := tprint(".%", kind);
    sql_text := tprint("#run orm_sql_text(T, %, where, %, %, %, %, %, %, Orm_Types, Orm_Relations)", kind_text, ifx kind == .SELECT then "order_by" else "where", ifx kind == .UPDATE then "set" else "where", desc, limit, offset, jai_string(from_sql));

    if kind == .SELECT print_to_builder(*builder, "orm_result, orm_error := query_as(T, db, %", sql_text);
    else print_to_builder(*builder, "orm_result, orm_error := execute(db, %", sql_text);

    for sql.parameters print_to_builder(*builder, ", %", orm_argument(*sql, it_index));

    append(*builder, ");\nreturn orm_result, orm_error;\n");

    return builder_to_string(*builder);
}

// A parameter's Jai text, inserted where the call was written.
orm_insert_text :: (text: string) -> string {
    return tprint("(#insert,scope(call) %)", jai_string(text));
}

// Parameter `index` as an argument: its text where the filter was written, through its wrap.
orm_argument :: (sql: *Orm_Sql, index: s64) -> string {
    text := orm_insert_text(sql.parameters[index]);
    wrap := sql.wraps[index];

    return ifx wrap then tprint("%(%)", wrap, text) else text;
}

orm_sql_text :: (T: Type, kind: Orm_Kind, where: Code, order_by: Code, set: Code, desc: bool, limit: s64, offset: s64, from_sql: string, types: [] Type, relations: [] Type) -> string {
    sql := orm_build(T, kind, where, order_by, set, desc, limit, offset, from_sql, types, relations);

    // Problems only the related structs reveal (an unknown field of `it.team`).
    if sql.problem {
        location := ifx sql.problem_node then node_location(sql.problem_node) else node_location(orm_root(where));
        compiler_report(sql.problem, location);
        return "";
    }

    return builder_to_string(*sql.builder);
}

// The body of `insert`.
orm_insert_code :: (T: Type) -> string {
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

    statement := ifx inserted then tprint("insert into % (%) values (%) returning %", table, builder_to_string(*names), builder_to_string(*holes), builder_to_string(*all)) else tprint("insert into % default values returning %", table, builder_to_string(*all));

    builder: String_Builder;
    builder.allocator = temp;

    print_to_builder(*builder, "rows, error := query_as(T, db, %1%2);\n", jai_string(statement), builder_to_string(*values));
    append(*builder, "row: T;\n");
    append(*builder, "if error.kind != .NONE return row, error;\n");
    append(*builder, "detached := Rows(T).{ result = detach_to_temp(*rows.result), map = rows.map };\n");
    append(*builder, "if detached.result.row_count != 1 return row, make_error(.USAGE, \"the insert returned % rows\", detached.result.row_count);\n");
    append(*builder, "error = read_row(*detached, 0, *row);\n");
    append(*builder, "return row, error;\n");

    return builder_to_string(*builder);
}
