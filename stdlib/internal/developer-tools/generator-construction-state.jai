#scope_module
#import "Basic";

gm_dependency_view_valid :: (view: $View) -> bool {
    return view.count >= 0 && view.count <= 0x7fffffffffffffff / size_of(view.data[0]) && (view.count == 0 || view.data != null);
}

// Temporary traversal state only; the Generator retains the sole canonical
// registry. A shared worklist covers type/expression cycles without recursion.
gm_declaration_depends_on :: (candidate: *Declaration, declaration: *Declaration) -> bool {
    parents: [..] *Declaration;
    types, seen_types: [..] *CType;
    expressions, seen_expressions: [..] *Expression;

    defer array_free(parents);
    defer array_free(types);
    defer array_free(seen_types);
    defer array_free(expressions);
    defer array_free(seen_expressions);

    parent := candidate;
    while parent {
        if parent == declaration return true;
        if array_find(parents, parent) break;
        array_add(*parents, parent);
        parent = parent.parent;
    }

    if candidate.kind == .STRUCT {
        record := cast(*Struct) candidate;
        if !gm_dependency_view_valid(record.declarations) || !gm_dependency_view_valid(record.parents) || !gm_dependency_view_valid(record.template_type_params) {
            gm_input_error("canonical dependency cleanup encountered malformed record storage");
            return true;
        }

        for member: record.declarations if member == declaration return true;

        for base: record.parents {
            if !base continue;
            if base == declaration return true;
            array_add(*types, base.type);
        }

        for parameter: record.template_type_params if parameter == declaration return true;
    }

    if candidate.kind == .ENUM {
        values := (cast(*Enum) candidate).enumerates;
        if !gm_dependency_view_valid(values) {
            gm_input_error("canonical dependency cleanup encountered malformed enum storage");
            return true;
        }

        for value: values if value == declaration return true;
    }

    array_add(*types, candidate.type);
    array_add(*expressions, candidate.expression);
    type_index, expression_index: int;
    while type_index < types.count || expression_index < expressions.count {
        if type_index < types.count {
            type := types[type_index];

            type_index += 1;
            if !type || array_find(seen_types, type) continue;

            array_add(*seen_types, type);
            if type.type_of_declaration == declaration || cast(*Declaration) type.type_of_struct == declaration ||
               cast(*Declaration) type.type_of_enum == declaration || cast(*Declaration) type.type_of_typedef == declaration return true;

            array_add(*types, type.pointer_to);
            array_add(*types, type.array_element_type);
            array_add(*expressions, type.array_element_count_expression);

            if !gm_dependency_view_valid(type.template_instantiation_params) {
                gm_input_error("canonical dependency cleanup encountered malformed template argument storage");
                return true;
            }

            for parameter: type.template_instantiation_params array_add(*expressions, parameter);

            if type.type_of_function {
                signature := type.type_of_function;
                if !gm_dependency_view_valid(signature.arguments) {
                    gm_input_error("canonical dependency cleanup encountered malformed function argument storage");
                    return true;
                }

                array_add(*types, signature.return_type);

                for argument: signature.arguments {
                    if !argument continue;
                    if argument == declaration return true;
                    array_add(*types, argument.type);
                    array_add(*expressions, argument.expression);
                }
            }

            continue;
        }

        expression := expressions[expression_index];

        expression_index += 1;
        if !expression || array_find(seen_expressions, expression) continue;

        array_add(*seen_expressions, expression);

        if expression.kind == .LITERAL {
            literal := cast(*Literal) expression;
            if !gm_dependency_view_valid(literal.template_instantiation_params) {
                gm_input_error("canonical dependency cleanup encountered malformed literal template argument storage");
                return true;
            }

            for parameter: literal.template_instantiation_params array_add(*expressions, parameter);
            if literal.literal_kind == .IDENTIFIER && literal.identifier == declaration return true;
            if literal.literal_kind == .ENUM && (cast(*Declaration) literal.enum_type == declaration || literal.enum_value == declaration) return true;
        } else if expression.kind == .TYPE_REFERENCE {
            array_add(*types, (cast(*Type_Reference) expression).type);
        } else if expression.kind == .PAREN_EXPRESSION {
            array_add(*expressions, (cast(*Paren_Expression) expression).child);
        } else if expression.kind == .UNARY_OPERATOR {
            array_add(*expressions, (cast(*Unary_Operator) expression).child);
        } else if expression.kind == .BINARY_OPERATOR {
            binary := cast(*Binary_Operator) expression;
            array_add(*expressions, binary.lhs);
            array_add(*expressions, binary.rhs);
        } else if expression.kind == .STRUCT_LITERAL {
            values := (cast(*Struct_Literal) expression).values;
            if !gm_dependency_view_valid(values) {
                gm_input_error("canonical dependency cleanup encountered malformed aggregate value storage");
                return true;
            }

            for value: values array_add(*expressions, value);
        }
    }

    return false;
}

gm_remove_rejected_members :: (members: *$Member_View, rejected: [] *Declaration) {
    if !gm_dependency_view_valid(members.*) {
        gm_input_error("canonical dependency cleanup encountered malformed member storage");
        return;
    }

    retained := 0;

    for member: members.* {
        if array_find(rejected, member) continue;
        members.data[retained] = member;
        retained += 1;
    }

    members.count = retained;
}

// Remove only the rejected canonical dependency closure, including recursive
// nominal shells and registered children. Independent reentrant construction
// stays canonical. Storage/native tokens remain owned by the SAME ledger.
gm_reject_construction :: (generator: *Generator, declaration: *Declaration) {
    rejected: [..] *Declaration;
    keys: [..] CXCursor;
    aliases: [..] string;

    defer array_free(rejected);
    defer array_free(keys);
    defer array_free(aliases);
    array_add(*rejected, declaration);
    changed := true;
    while changed {
        changed = false;

        for entry: generator.declarations_by_cursor.entries {
            if entry.hash < GM_Table.FIRST_VALID_HASH || !entry.value || array_find(rejected, entry.value) continue;

            for target: rejected {
                if !gm_declaration_depends_on(entry.value, target) continue;
                array_add(*rejected, entry.value);
                changed = true;
                break;
            }
        }
    }

    for entry: generator.declarations_by_cursor.entries {
        if entry.hash >= GM_Table.FIRST_VALID_HASH && array_find(rejected, entry.value) array_add(*keys, entry.key);
    }

    for key: keys GM_Table.table_remove(*generator.declarations_by_cursor, key);

    for entry: generator.unwrapped_typedefs.entries {
        if entry.hash >= GM_Table.FIRST_VALID_HASH && array_find(rejected, cast(*Declaration) entry.value) array_add(*aliases, entry.key);
    }

    for key: aliases GM_Table.table_remove(*generator.unwrapped_typedefs, key);

    session := gm_current_session();

    for target: rejected {
        phase := gm_scope_index(session, target);
        if phase >= 0 session.scope_states[phase].stage = .REJECTED;
        if target.kind == .STRUCT GM_Table.table_remove(*generator.extra_infos, cast(*Struct) target);
    }

    for entry: generator.declarations_by_cursor.entries {
        if entry.hash < GM_Table.FIRST_VALID_HASH || !entry.value || entry.value.kind != .NAMESPACE continue;
        gm_remove_rejected_members(*(cast(*Namespace) entry.value).declarations, rejected);
    }

    gm_remove_rejected_members(*generator.global_scope.members, rejected);
    gm_remove_rejected_members(*generator.flattened_global_scope_members, rejected);
}
