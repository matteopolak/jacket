compiler_get_nodes :: (code: Code) -> (root: *C.Code_Node, expressions: [] *C.Code_Node) #compiler {
    initialize_records();
    id := __jaic_code_nodes(code);
    nodes := cast(*Code_Nodes) __jaic_rec_build(record_builder, id, type_info(Code_Nodes));

    require(nodes != null, "compiler_get_nodes could not materialize its record");

    return nodes.root, nodes.expressions;
}

compiler_get_code :: (node: *C.Code_Node, code_to_copy_scope_from: Code = #code,null) -> Code #compiler {
    Source_Print :: #import "internal/program-print";

    require(node != null, "compiler_get_code requires a syntax node");
    builder: Basic.String_Builder;

    Basic.init_string_builder(*builder);
    // The root may be a statement (for example, a compound assignment).
    // Parenthesizing that root would force expression parsing on reconstruction.
    success := Source_Print.expression(*builder, node, true, true, 0);

    require(success, "compiler_get_code cannot print this syntax node kind");
    text := Basic.builder_to_string(*builder);
    result := __jaic_parse_code(text, code_to_copy_scope_from);

    Basic.free(text);
    Basic.free_buffers(*builder);

    return result;
}

get_root_type :: (code: Code) -> (status: C.Get_Root_Type_Status, type: Type) #compiler {
    if __jaic_code_is_null(code) return .INPUT_IS_CODE_NULL, void;

    root := compiler_get_nodes(code);
    if !root return .CORRUPTED, void;
    if !root.type return .NOT_TYPED, void;

    return .SUCCESS, get_type(root.type);
}

get_type :: (ti: *Type_Info) -> Type #compiler {
    return cast(Type) ti;
}

get_type_table :: (w: Workspace = -1) -> [] *Type_Info {
    if !#compile_time return get_runtime_info(w).type_table;
    result: [] *Type_Info;
    __jaic_type_table(resolve_workspace(w), *result);
    return result;
}

compiler_make_procedure_live :: (w: Workspace, header: *C.Code_Procedure_Header) #compiler {
    __jaic_make_procedure_live(resolve_workspace(w), record_id(header));
}

compiler_modify_procedure :: (w: Workspace, body: *C.Code_Procedure_Body) #compiler {
    Source_Print :: #import "internal/program-print";

    require(body && body.block, "compiler_modify_procedure requires a procedure body and block");
    ids: [..] s64;
    sources: [..] string;

    for node: body.block.statements {
        builder: Basic.String_Builder;

        Basic.init_string_builder(*builder);
        // Serialize every current statement, including edits to existing records;
        // retaining the old record would silently discard those edits.
        success := Source_Print.expression(*builder, node, true, true, 0);

        require(success, "compiler_modify_procedure cannot print an edited statement kind");
        Basic.array_add(*ids, cast(s64) 0);
        Basic.array_add(*sources, Basic.builder_to_string(*builder));
        Basic.free_buffers(*builder);
    }

    __jaic_modify_procedure(resolve_workspace(w), record_id(body), ids.data, sources.data, ids.count);
    for sources Basic.free(it);

    Basic.array_free(ids);
    Basic.array_free(sources);
}

#scope_module

Record_Plan_Entry :: struct {
    offset: u64;
    kind: u64;
    size: u64;
    name: string;
    type: *Type_Info;
    element_kind: u64;
    element_size: u64;
    target: *Type_Info;
    resizable: u64;
}

Code_Nodes :: struct {
    root: *C.Code_Node;
    expressions: [] *C.Code_Node;
}

record_builder: u64;
planned_types: [..] *Type_Info;

record_id :: (pointer: *void) -> s64 {
    initialize_records();
    result := __jaic_rec_of(record_builder, pointer);
    require(result != 0, "syntax/message pointer was not produced by this compiler record builder");
    return result;
}

initialize_records :: () {
    if record_builder != 0 return;

    record_builder = __jaic_rec_builder();
    register_type(type_info(Code_Nodes));
    register_compiler_types();
    __jaic_rec_builtin(record_builder, "s8", type_info(s8));
    __jaic_rec_builtin(record_builder, "s16", type_info(s16));
    __jaic_rec_builtin(record_builder, "s32", type_info(s32));
    __jaic_rec_builtin(record_builder, "s64", type_info(s64));
    __jaic_rec_builtin(record_builder, "u8", type_info(u8));
    __jaic_rec_builtin(record_builder, "u16", type_info(u16));
    __jaic_rec_builtin(record_builder, "u32", type_info(u32));
    __jaic_rec_builtin(record_builder, "u64", type_info(u64));
    __jaic_rec_builtin(record_builder, "float32", type_info(float32));
    __jaic_rec_builtin(record_builder, "float64", type_info(float64));
    __jaic_rec_builtin(record_builder, "bool", type_info(bool));
    __jaic_rec_builtin(record_builder, "string", type_info(string));
    __jaic_rec_builtin(record_builder, "void", type_info(void));
    __jaic_rec_builtin(record_builder, "Type", type_info(Type));
    __jaic_rec_builtin(record_builder, "Code", type_info(Code));
    __jaic_rec_builtin(record_builder, "Any", type_info(Any));
}

representation :: (info: *Type_Info) -> *Type_Info {
    while info && info.type == .VARIANT info = (cast(*Type_Info_Variant) info).variant_of;
    return info;
}

plan_kind :: (info: *Type_Info) -> u64 {
    info = representation(info);
    if !info return 0;
    if info.type == .STRING return 2;
    if info.type == .POINTER return 3;
    if info.type == .STRUCT return 4;
    if info.type == .ARRAY return 5;
    if info.type == .INTEGER || info.type == .BOOL || info.type == .FLOAT || info.type == .ENUM || info.type == .TYPE || info.type == .CODE return 1;

    return 0;
}

register_type :: (info: *Type_Info) {
    info = representation(info);
    if !info || info.type != .STRUCT return;
    for planned_types if it == info return;

    Basic.array_add(*planned_types, info);
    s := cast(*Type_Info_Struct) info;

    __jaic_rec_type(record_builder, s.name, info);
    entries: [..] Record_Plan_Entry;

    describe_members(s, 0, *entries);
    __jaic_rec_plan(record_builder, info, cast(u64) info.runtime_size, entries.data, cast(u64) entries.count, s.name == "Type_Info_Struct");
    Basic.array_free(entries);
}

describe_members :: (s: *Type_Info_Struct, base: u64, entries: *[..] Record_Plan_Entry) {
    for member: s.members {
        if member.flags & .CONSTANT continue;

        t := representation(member.type);
        if !t continue;

        offset := base + cast(u64) member.offset_in_bytes;

        if member.flags & .USING && t.type == .STRUCT {
            describe_members(cast(*Type_Info_Struct) t, offset, entries);
            continue;
        }

        entry: Record_Plan_Entry;
        entry.offset = offset;
        entry.kind = plan_kind(t);
        entry.size = cast(u64) t.runtime_size;
        entry.name = member.name;
        entry.type = t;

        if t.type == .STRUCT {
            entry.target = t;
            register_type(t);
        } else if t.type == .POINTER {
            entry.target = (cast(*Type_Info_Pointer) t).pointer_to;
            register_type(entry.target);
        } else if t.type == .ARRAY {
            a := cast(*Type_Info_Array) t;

            require(a.array_type != .FIXED, "fixed-array compiler record fields need an explicit serialization plan");
            element := representation(a.element_type);

            entry.element_kind = plan_kind(element);
            entry.element_size = cast(u64) element.runtime_size;

            if element.type == .STRUCT {
                entry.target = element;
                register_type(element);
            } else if element.type == .POINTER {
                entry.target = (cast(*Type_Info_Pointer) element).pointer_to;
                register_type(entry.target);
            }

            if a.array_type == .RESIZABLE entry.resizable = 1;
        }

        Basic.array_add(entries, entry);
    }
}

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
