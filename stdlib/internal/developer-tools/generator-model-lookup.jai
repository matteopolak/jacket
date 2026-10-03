#scope_module
#import "Basic";
#load "generator-record-metadata.jai";

gm_first_member_offset :: (outer: CXType, record: *Struct, path: *[..] *Struct) -> int {
    if !record return -1;

    if array_find(path.*, record) {
        gm_construction_error("cyclic anonymous record while searching field offset");
        return -1;
    }

    array_add(path, record);
    defer path.count -= 1;

    for member: record.declarations {
        if !member continue;

        nested: *Struct;
        if member.kind == .STRUCT && ((cast(*Struct) member).flags & .IS_ANON) != 0 nested = cast(*Struct) member;

        if member.kind == .DECLARATION && member.storage_class != .Static && (member.decl_flags & .IS_METADATA) == 0 {
            if member.name {
                name := to_c_string(member.name);
                defer free(name);
                return clang_Type_getOffsetOf(outer, name);
            }

            type := gm_underlying(member.type);
            if type nested = type.type_of_struct;
        }

        if nested {
            offset := gm_first_member_offset(outer, nested, path);
            if offset != -1 return offset;
        }
    }

    return -1;
}

gm_macro_enum :: (name: string, declaration: *Declaration) -> *Enum, bool {
    generator := gm_active_generator();
    session := gm_current_session();

    if !generator || !session return null, false;
    for candidate: generator.enum_macros if candidate && (candidate.name == name || candidate.output_name == name) return candidate, false;
    for candidate: generator.global_scope.members if candidate && candidate.kind == .ENUM && (candidate.name == name || candidate.output_name == name) return cast(*Enum) candidate, false;

    result := GM_Sessions.new_owned(session, Enum);
    result.kind = .ENUM;
    result.name = GM_Sessions.copy_text(session, name);
    result.output_name = GM_Sessions.copy_text(session, name);

    if declaration {
        result.location = declaration.location;
        result.comment.location = declaration.comment.location;
        result.comment.text = GM_Sessions.copy_text(session, declaration.comment.text);
    }

    if generator.enum_macros.allocated == 0 GM_Sessions.append_owned(session, *generator.enum_macros, result);
    else array_add(*generator.enum_macros, result);

    return result, true;
}

gm_enum_identifier :: (name: string, enumeration: *Enum, ignore_omit: bool) -> *Declaration {
    if !enumeration return null;

    for value: enumeration.enumerates {
        if !value || (!ignore_omit && (value.decl_flags & .OMIT_FROM_OUTPUT) != 0) continue;
        if value.name == name || value.output_name == name return value;
    }

    return null;
}

gm_enum_identifier_type :: (enumeration: *Enum) -> *CType {
    session := gm_current_session();
    if !session return null;

    result := GM_Sessions.new_owned(session, CType);
    result.type_of_enum = enumeration;

    if enumeration.type {
        result.size = enumeration.type.size;
        result.alignment = enumeration.type.alignment;
        result.number_flags = enumeration.type.number_flags;
    }

    return result;
}

gm_macro_identifier :: (name: string, enumeration: *Enum, ignore_omit: bool) -> bool, string, *CType, *Enum {
    generator := gm_active_generator();
    if !generator return false, "", null, null;
    if name == "void" return generator.type_def_void != null, "void", generator.type_def_void, null;
    if name == "int" return generator.type_def_s32 != null, "s32", generator.type_def_s32, null;

    value := gm_enum_identifier(name, enumeration, ignore_omit);
    if value return true, gm_name(value), gm_enum_identifier_type(enumeration), enumeration;

    found, converted_enum := GM_Table.table_find(*generator.macros_converted_to_enum_values, name);
    if found && converted_enum {
        value = gm_enum_identifier(name, converted_enum, ignore_omit);
        if value return true, gm_name(value), gm_enum_identifier_type(converted_enum), converted_enum;
    }

    found_alias, alias := GM_Table.table_find(*generator.unwrapped_typedefs, name);
    if found_alias && alias && alias.type {
        underlying := gm_underlying(alias.type);
        if underlying && underlying.type_of_struct return true, gm_name(cast(*Declaration) underlying.type_of_struct), alias.type, null;
        if underlying && underlying.type_of_enum return true, gm_name(cast(*Declaration) underlying.type_of_enum), alias.type, underlying.type_of_enum;
        return true, gm_name(cast(*Declaration) alias), alias.type, null;
    }

    for candidate: generator.global_scope.members {
        if !candidate || (candidate.name != name && candidate.output_name != name) continue;
        if !ignore_omit && (candidate.decl_flags & .OMIT_FROM_OUTPUT) != 0 continue;

        if candidate.kind == .MACRO_DEFINITION {
            if candidate.macro_value_result == .NEEDS_CHECKING {
                #if #exists(check_macro_value) {
                    check_macro_value(candidate);
                } else {
                    gm_construction_error("pending macro validation requires the tooling-owned check_macro_value implementation");
                    return false, "", null, null;
                }
            }

            if candidate.macro_value_result != .KEEP continue;

            if candidate.should_unwrap {
                session := gm_current_session();
                if !session return false, "", null, null;

                if session.macro_lookup_depth >= 128 {
                    gm_construction_error("macro value substitution exceeds supported nesting depth");
                    return false, "", null, null;
                }

                session.macro_lookup_depth += 1;
                defer session.macro_lookup_depth -= 1;

                if !candidate.expression || candidate.expression.kind != .LITERAL || (cast(*Literal) candidate.expression).literal_kind != .MACRO {
                    gm_construction_error("unwrapped macro lookup requires canonical replacement-token storage");
                    return false, "", null, null;
                }

                converted := string_from_macro_value((cast(*Literal) candidate.expression).macro_value, name = candidate.name);

                defer free(converted);
                if !converted return false, "", null, null;

                return true, GM_Sessions.copy_text(session, converted), candidate.type, null;
            }
        }

        parent_enum: *Enum;
        if candidate.parent && candidate.parent.kind == .ENUM parent_enum = cast(*Enum) candidate.parent;

        return true, gm_name(candidate), candidate.type, parent_enum;
    }

    return false, "", null, null;
}

gm_extra_info :: (type: *CType, base: *Declaration) -> *Struct_Extra_Info, *Struct {
    if type && type.type_of_declaration && (type.type_of_declaration.decl_flags & .TEMPLATE) != 0 return *struct_extra_info_unknown, null;
    if base && (base.decl_flags & .TEMPLATE) != 0 && base.kind != .STRUCT return *struct_extra_info_unknown, null;

    canonical := gm_underlying(type);
    if !canonical || !canonical.type_of_struct || (base && base.kind != .STRUCT) {
        gm_construction_error("record metadata requires a represented record parent");
        return null, null;
    }

    record := canonical.type_of_struct;

    generator := gm_active_generator();

    if !generator return null, null;

    session := context._bindings_generator_session;
    if session {
        phase := gm_scope_index(session, cast(*Declaration) record);
        if phase >= 0 && session.scope_states[phase].stage == .REJECTED {
            gm_construction_error("record metadata cannot use a rejected canonical dependency");
            return null, null;
        }

        if phase >= 0 && session.scope_states[phase].stage != .FULL return *struct_extra_info_unknown, null;
    }

    if (record.decl_flags & .TEMPLATE) != 0 || record.template_type_params.count > 0 return *struct_extra_info_unknown, null;

    cached := GM_Table.table_find_pointer(*generator.extra_infos, record);
    if cached return cached, record;
    if canonical.size < 0 || canonical.alignment <= 0 return *struct_extra_info_unknown, null;
    if (record.flags & (.OBJC_INTERFACE | .OBJC_PROTOCOL | .OBJC_CATEGORY)) != 0 return *struct_extra_info_unknown, null;
    if !gm_current_session() return null, null;

    info := gm_record_metadata(record);

    return info, ifx info then record else null;
}
