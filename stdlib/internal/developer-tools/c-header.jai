handle_message :: (using info: *Header_Info, message: *Message) {
    if !message || message.kind != .TYPECHECKED return;

    typed := cast(*Message_Typechecked) message;
    for item: typed.structs if item.expression && item.expression.defined_type header_record_support(info, item.expression.defined_type);

    for item: typed.declarations {
        decl := item.expression;
        if !decl || !(decl.flags & .PROGRAM_EXPORT) || decl.name == "main" continue;

        if decl.type && decl.type.type == .PROCEDURE {
            if !array_find(exported_procedures, decl) array_add(*exported_procedures, decl);
        } else if decl.expression {
            ti: *Type_Info;
            if decl.expression.kind == .TYPE_DEFINITION ti = (cast(*Code_Type_Definition) decl.expression).info;
            if decl.expression.kind == .STRUCT ti = (cast(*Code_Struct) decl.expression).defined_type;

            if ti {
                name := decl.program_export_name;
                if !name name = decl.name;

                table_set(*exported_types, ti, name);
                table_set(*name_types, name, ti);
                if !array_find(ordered_types, ti) array_add(*ordered_types, ti);
            } else {
                log_error("C header: exported declaration % has no supported type definition", decl.name);
            }
        }
    }
}

generate_header :: (using info: *Header_Info, path: string) -> bool {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, "/* Generated C ABI declarations. */\n#pragma once\n#include <stdint.h>\n#include <stddef.h>\n#ifndef __cplusplus\n#if !defined(__STDC_VERSION__) || __STDC_VERSION__ < 202311L\ntypedef _Bool bool;\n#define false ((bool)0)\n#define true ((bool)1)\n#endif\n#define JAI_HEADER_ASSERT _Static_assert\n#define JAI_HEADER_ALIGNOF _Alignof\n#else\n#define JAI_HEADER_ASSERT static_assert\n#define JAI_HEADER_ALIGNOF alignof\nextern \"C\" {\n#endif\n");

    for width: 0..3 {
        bits := cast(s64) 8 << width;
        append(*builder, tprint("typedef int%_t s%;\ntypedef uint%_t u%;\n", bits, bits, bits, bits));
    }

    types: [..] *Type_Info;

    defer array_free(types);
    for ti: ordered_types if !array_find(types, ti) array_add(*types, ti);
    for entry: exported_types.entries if entry.hash >= FIRST_VALID_HASH && !array_find(types, entry.key) array_add(*types, entry.key);

    for decl: exported_procedures if header_emitted_procedure(decl) {
        for arg: (cast(*Type_Info_Procedure) decl.type).argument_types header_add_type(*types, arg);
        for result: (cast(*Type_Info_Procedure) decl.type).return_types header_add_type(*types, result);
    }

    index: s64;
    while index < types.count {
        ti := types[index];
        if !ti return header_error("null exported type");
        if ti.type == .POINTER header_add_type(*types, (cast(*Type_Info_Pointer) ti).pointer_to);

        support := header_support_type(info, ti);

        if header_needs_support(ti) {
            if !support return header_error("missing compiler/runtime support descriptor");
            header_add_type(*types, support);
        }

        if ti.type == .ARRAY && (cast(*Type_Info_Array) ti).array_type == .FIXED header_add_type(*types, (cast(*Type_Info_Array) ti).element_type);
        if ti.type == .VARIANT header_add_type(*types, (cast(*Type_Info_Variant) ti).variant_of);

        if ti.type == .STRUCT && !header_polymorphic(ti) {
            for member: (cast(*Type_Info_Struct) ti).members if !header_omitted_member(member) header_add_type(*types, member.type);
        }

        index += 1;
    }

    // Forward tags break pointer cycles without inventing a by-value order.
    for ti: types {
        if !ti return header_error("null exported type");

        if ti.type == .STRUCT && !header_polymorphic(ti) {
            name, ok := header_type_name(info, ti);
            if !ok return false;

            st := cast(*Type_Info_Struct) ti;
            tag := ifx st.textual_flags & .UNION then "union" else "struct";

            append(*builder, tprint("typedef % % %;\n", tag, name, name));
        }
    }

    done: [..] *Type_Info;
    active: [..] *Type_Info;

    defer array_free(done);
    defer array_free(active);
    for ti: types if !header_define_type(info, *builder, ti, *done, *active) return false;

    for decl: exported_procedures {
        if !header_emitted_procedure(decl) continue;

        name := decl.program_export_name;
        if !name name = decl.name;
        if !header_identifier(name) return header_error("invalid exported procedure name");
        if !decl.type || decl.type.type != .PROCEDURE return header_error("exported procedure has no procedure descriptor");

        signature, ok := header_declarator(info, decl.type, name, true);
        if !ok return false;

        append(*builder, tprint("extern %;\n", signature));
    }

    append(*builder, "#ifdef __cplusplus\n}\n#endif\n#undef JAI_HEADER_ASSERT\n#undef JAI_HEADER_ALIGNOF\n");
    if builder.failed return header_error("output allocation failed");
    if !write_entire_file(path, *builder, false) return header_error("could not write header");

    return true;
}

#scope_file
#import "Basic";
#import "Compiler";
#import "Hash_Table";
#import "File";
#import "String";

header_record_support :: (info: *Header_Info, ti: *Type_Info_Struct) {
    if ti.name == "Resizable_Array" info.type_resizable_array = ti;
    if ti.name == "Array_View_8" info.type_array_view_8 = ti;
    if ti.name == "Array_View_16" info.type_array_view_16 = ti;
    if ti.name == "Array_View_32" info.type_array_view_32 = ti;
    if ti.name == "Array_View_64" info.type_array_view_64 = ti;
    if ti.name == "Type_Info" info.type_type_info = ti;
    if ti.name == "Newstring" info.type_newstring = ti;
    if ti.name == "Any_Struct" info.type_any = ti;
}

header_emitted_procedure :: (decl: *Code_Declaration) -> bool {
    if !decl || decl.name == "main" || !decl.type || decl.type.type != .PROCEDURE return false;
    return ((cast(*Type_Info_Procedure) decl.type).procedure_flags & .IS_C_CALL) != 0;
}

header_polymorphic :: (ti: *Type_Info) -> bool {
    return ti && ti.type == .STRUCT && ((cast(*Type_Info_Struct) ti).nontextual_flags & .POLYMORPHIC) != 0;
}

header_omitted_member :: (member: Type_Info_Struct_Member) -> bool {
    return (member.flags & (.CONSTANT | .IMPORTED)) != 0 || (member.type && member.type.type == .VOID);
}

header_needs_support :: (ti: *Type_Info) -> bool {
    if ti.type == .STRING || ti.type == .ANY || ti.type == .TYPE return true;
    return ti.type == .ARRAY && (cast(*Type_Info_Array) ti).array_type != .FIXED;
}

header_support_type :: (info: *Header_Info, ti: *Type_Info) -> *Type_Info {
    if ti.type == .STRING return info.type_newstring;
    if ti.type == .ANY return info.type_any;
    if ti.type == .TYPE return info.type_type_info;

    if ti.type == .ARRAY {
        array := cast(*Type_Info_Array) ti;
        if array.array_type == .VIEW return info.type_array_view_64;
        if array.array_type == .RESIZABLE return info.type_resizable_array;
    }

    return null;
}

header_add_type :: (types: *[..] *Type_Info, ti: *Type_Info) {
    if ti && !array_find(types.*, ti) array_add(types, ti);
}

header_builder_text :: (builder: *String_Builder) -> string {
    push_allocator(temp);
    return builder_to_string(builder, do_reset = false);
}

header_error :: (reason: string) -> bool {
    log_error("C header: %", reason);
    return false;
}

header_identifier :: (name: string) -> bool {
    if !name return false;

    for c, index: name {
        if (c >= #char "a" && c <= #char "z") || (c >= #char "A" && c <= #char "Z") || c == #char "_" continue;
        if index > 0 && c >= #char "0" && c <= #char "9" continue;
        return false;
    }

    // Reject C11 keywords rather than silently renaming ABI symbols.
    keywords := " auto break case char const continue default do double else enum extern float for goto if inline int long register restrict return short signed sizeof static struct switch typedef union unsigned void volatile while _Alignas _Alignof _Atomic _Bool _Complex _Generic _Imaginary _Noreturn _Static_assert _Thread_local ";
    needle := tprint(" % ", name);

    for index: 0..keywords.count - needle.count {
        if slice(keywords, index, needle.count) == needle return false;
    }

    return true;
}

header_type_name :: (info: *Header_Info, ti: *Type_Info) -> string, bool {
    found, name := table_find(*info.exported_types, ti);
    if !found {
        if ti.type == .STRUCT name = (cast(*Type_Info_Struct) ti).name;
        else if ti.type == .ENUM name = (cast(*Type_Info_Enum) ti).name;
        else if ti.type == .VARIANT name = (cast(*Type_Info_Variant) ti).name;
    }

    raw := name;
    candidate := tprint("%%", info.jai_type_prefix, raw);
    suffix: s64;

    while true {
        occupied, previous := table_find(*info.name_types, candidate);
        if !occupied || previous == ti break;

        suffix += 1;
        name = tprint("%%", raw, suffix);
        candidate = tprint("%%", info.jai_type_prefix, name);
    }

    if !header_identifier(candidate) return "", header_error("missing or invalid C type name");

    if !table_contains(*info.name_types, candidate) {
        owner := ifx info.name_types.allocator.proc then info.name_types.allocator else context.allocator;
        table_add(*info.name_types, header_persist_name(candidate, owner), ti);
    }

    previous_found, previous_name := table_find(*info.exported_types, ti);
    if !previous_found || previous_name != name {
        owner := ifx info.exported_types.allocator.proc then info.exported_types.allocator else context.allocator;
        table_set(*info.exported_types, ti, header_persist_name(name, owner));
    }

    return candidate, true;
}

header_persist_name :: (name: string, owner: Allocator) -> string {
    push_allocator(owner);
    return copy_string(name);
}

header_declarator :: (info: *Header_Info, ti: *Type_Info, name: string, function_declaration: bool) -> string, bool {
    if !ti return "", header_error("missing reflected type");

    if header_needs_support(ti) {
        support := header_support_type(info, ti);
        if !support return "", header_error("missing compiler/runtime support descriptor");

        projected_name, ok := header_type_name(info, support);
        if !ok return "", false;

        return tprint("% %%", projected_name, ifx ti.type == .TYPE then "*" else "", name), true;
    }

    base: string;
    if ti.type == .VOID base = "void";
    else if ti.type == .BOOL base = "bool";
    else if ti.type == .INTEGER {
        if ti.runtime_size != 1 && ti.runtime_size != 2 && ti.runtime_size != 4 && ti.runtime_size != 8 return "", header_error("unsupported C integer width");
        base = tprint("%%", ifx (cast(*Type_Info_Integer) ti).signed then "s" else "u", ti.runtime_size * 8);
    } else if ti.type == .FLOAT {
        if ti.runtime_size == 4 base = "float";
        else if ti.runtime_size == 8 base = "double";
        else return "", header_error("unsupported C floating width");
    } else if ti.type == .POINTER {
        pointer := cast(*Type_Info_Pointer) ti;
        nested := tprint("*%", name);
        if pointer.pointer_to && pointer.pointer_to.type == .ARRAY nested = tprint("(*%)", name);
        return header_declarator(info, pointer.pointer_to, nested, false);
    } else if ti.type == .ARRAY {
        array := cast(*Type_Info_Array) ti;
        if array.array_type != .FIXED return "", header_error("array views and resizable arrays require a C storage projection");
        if array.array_count <= 0 return "", header_error("C does not support zero-sized fixed arrays");
        return header_declarator(info, array.element_type, tprint("%[%]", name, array.array_count), false);
    } else if ti.type == .PROCEDURE {
        if !function_declaration return tprint("void *%", name), true;

        procedure := cast(*Type_Info_Procedure) ti;
        if !(procedure.procedure_flags & .IS_C_CALL) return "", header_error("procedure must have the C calling convention");
        if procedure.return_types.count > 1 return "", header_error("C cannot project multiple return values");

        args: String_Builder;

        defer free_buffers(*args);

        for arg, index: procedure.argument_types {
            if index > 0 append(*args, ", ");
            if (arg.type == .ARRAY && (cast(*Type_Info_Array) arg).array_type == .FIXED) || arg.type == .VOID return "", header_error("C argument adjustment would change the Jai ABI");

            text, ok := header_declarator(info, arg, tprint("arg%", index), false);
            if !ok return "", false;

            append(*args, text);
        }

        if procedure.argument_types.count == 0 append(*args, "void");

        declarator := ifx function_declaration then name else tprint("(*%)", name);
        fn := tprint("%(%)", declarator, header_builder_text(*args));

        if procedure.return_types.count == 0 return tprint("void %", fn), true;

        result := procedure.return_types[0];
        if result.type == .ARRAY && (cast(*Type_Info_Array) result).array_type == .FIXED return "", header_error("invalid C return type");

        return header_declarator(info, result, fn, false);
    } else if ti.type == .STRUCT || ti.type == .ENUM || ti.type == .VARIANT {
        ok: bool;
        base, ok = header_type_name(info, ti);
        if !ok return "", false;
    } else return "", header_error("unsupported reflected C ABI type");

    return tprint("% %", base, name), true;
}

header_define_type :: (info: *Header_Info, builder: *String_Builder, ti: *Type_Info, done: *[..] *Type_Info, active: *[..] *Type_Info) -> bool {
    if !ti return header_error("missing reflected type");
    if array_find(done.*, ti) return true;
    if array_find(active.*, ti) return header_error("cyclic by-value type dependency");

    array_add(active, ti);
    defer active.count -= 1;
    if ti.type == .POINTER return true;
    if header_needs_support(ti) return header_define_type(info, builder, header_support_type(info, ti), done, active);
    if ti.type == .ARRAY return header_define_type(info, builder, (cast(*Type_Info_Array) ti).element_type, done, active);
    if header_polymorphic(ti) return true;
    if ti.type != .STRUCT && ti.type != .ENUM && ti.type != .VARIANT return true;

    name, valid := header_type_name(info, ti);
    if !valid return false;

    if ti.type == .STRUCT {
        st := cast(*Type_Info_Struct) ti;
        if st.runtime_size <= 0 || st.alignment <= 0 return header_error("incomplete or empty struct");
        if st.textual_flags & (.UNION_IS_TAGGED | .NO_PADDING) return header_error("packed and tagged-union projections are not supported");

        for member: st.members {
            if header_omitted_member(member) continue;
            if member.flags & .OVERLAY return header_error("overlays require an explicit C union projection");
            if !header_define_type(info, builder, member.type, done, active) return false;
        }

        tag := ifx st.textual_flags & .UNION then "union" else "struct";

        append(builder, tprint("% % {\n", tag, name));

        for member: st.members {
            if header_omitted_member(member) continue;
            if !header_identifier(member.name) return header_error("invalid C member name");

            text, ok := header_declarator(info, member.type, member.name, false);
            if !ok return false;

            append(builder, tprint("    %;\n", text));
        }

        append(builder, "};\n");
        append(builder, tprint("JAI_HEADER_ASSERT(sizeof(%) == %, \"Jai size mismatch\");\n", name, st.runtime_size));
        append(builder, tprint("JAI_HEADER_ASSERT(JAI_HEADER_ALIGNOF(%) == %, \"Jai alignment mismatch\");\n", name, st.alignment));

        for member: st.members {
            if header_omitted_member(member) continue;
            append(builder, tprint("JAI_HEADER_ASSERT(offsetof(%, %) == %, \"Jai field offset mismatch\");\n", name, member.name, member.offset_in_bytes));
        }
    } else if ti.type == .ENUM {
        en := cast(*Type_Info_Enum) ti;
        if en.names.count != en.values.count return header_error("invalid enum descriptor");

        if info.enum_internal_types {
            base, ok := header_declarator(info, en.internal_type, "", false);
            if !ok return false;
            append(builder, tprint("typedef enum % : % {\n", name, base));
        } else append(builder, tprint("typedef enum % {\n", name));

        prefix: String_Builder;

        defer free_buffers(*prefix);
        for c: name if c >= #char "A" && c <= #char "Z" append(*prefix, string.{ 1, *c });

        for item, index: en.names {
            symbol := ifx info.prefix_enum_values then tprint("%_%", header_builder_text(*prefix), item) else item;
            if !header_identifier(symbol) return header_error("invalid C enum value name");
            if !en.internal_type.signed && en.values[index] < 0 append(builder, tprint("    % = %ULL,\n", symbol, cast(u64) en.values[index]));
            else append(builder, tprint("    % = %,\n", symbol, en.values[index]));
        }

        append(builder, tprint("} %;\n", name));
    } else {
        variant := cast(*Type_Info_Variant) ti;
        if !header_define_type(info, builder, variant.variant_of, done, active) return false;

        text, ok := header_declarator(info, variant.variant_of, name, false);
        if !ok return false;

        append(builder, tprint("typedef %;\n", text));
    }

    append(builder, "\n");
    array_add(done, ti);

    return true;
}
