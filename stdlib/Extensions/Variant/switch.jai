// variant_switch: one handler per case, checked at compile time.
//
// A handler is a procedure taking a pointer to its case's type: `(it: *s64) { ... }`. They are
// matched to cases by that type, in any order, so the call reads like a switch. Jai has no
// procedure that takes a different type of argument for each of any number of positions, so the
// procedures below are written out for one to MAX_SWITCH_CASES handlers.

#scope_export

MAX_SWITCH_CASES :: 16;

#insert -> string {
    return switch_overloads();
};

#scope_module

// Runs `handler` on the payload, which `P` is the type of.
call_handler :: (handler: (*$P), payload: *void) {
    handler(cast(*P) payload);
}

// The case type a handler takes a pointer to, or null when it is not `(*T)`.
handler_case :: (info: *Type_Info) -> *Type_Info {
    if info.type != .PROCEDURE return null;

    procedure := cast(*Type_Info_Procedure) info;

    if procedure.argument_types.count != 1 || procedure.return_types.count != 0 return null;

    argument := procedure.argument_types[0];

    if argument.type != .POINTER return null;

    return (cast(*Type_Info_Pointer) argument).pointer_to;
}

// The statements of a variant_switch: for each handler, run it when its case is live. Reports a
// missing, repeated or foreign case and then returns nothing.
switch_body :: ($V: Type, handlers: [] *Type_Info) -> string {
    #if !#run is_variant_type(type_info(V)) {
        compiler_report(tprint("variant_switch: `%` is not a Variant", V));

        return "";
    } else {
        cases := V.types;
        ok := true;
        covered: [..] bool;

        for cases array_add(*covered, false);

        builder: String_Builder;

        for handler, handler_index: handlers {
            case_info := handler_case(handler);

            if !case_info {
                compiler_report(tprint("variant_switch: handler % must be a procedure with one pointer argument and no result, such as `(it: *%) { ... }`", handler_index + 1, cases[0]));
                ok = false;

                continue;
            }

            position := -1;

            for cases if type_info(it) == case_info position = it_index;

            if position < 0 {
                compiler_report(tprint("variant_switch: handler % takes a pointer to a type that is not a case of `%`. The cases are %", handler_index + 1, variant_name(V), type_list_text(cases)));
                ok = false;

                continue;
            }

            if covered[position] {
                compiler_report(tprint("variant_switch: `%` has more than one handler for the case `%`", variant_name(V), cases[position]));
                ok = false;

                continue;
            }

            covered[position] = true;
            print(*builder, "if v.index == % { call_handler(f%, v.payload.data); return; }\n", position, handler_index + 1);
        }

        for covered if !it {
            compiler_report(tprint("variant_switch: `%` has no handler for the case `%`", variant_name(V), cases[it_index]));
            ok = false;
        }

        if !ok return "";

        return builder_to_string(*builder,, temp);
    }
}

// The source of the overloads: for each count, one taking a pointer to the variant (handlers see
// the live payload in place) and one taking it by value (handlers see a copy).
switch_overloads :: () -> string {
    builder: String_Builder;

    for count: 1..MAX_SWITCH_CASES {
        parameters: String_Builder;
        arguments: String_Builder;
        infos: String_Builder;

        for 1..count {
            print(*parameters, ", f%: $F%", it, it);
            print(*arguments, ", f%", it);
            print(*infos, "%type_info(F%)", ifx it > 1 then ", " else "", it);
        }

        parameter_text := builder_to_string(*parameters,, temp);
        argument_text := builder_to_string(*arguments,, temp);
        info_text := builder_to_string(*infos,, temp);

        print(*builder, "variant_switch :: (v: *$V/Variant%) {\n", parameter_text);
        print(*builder, "    #insert -> string {\n        handlers: [] *Type_Info = .[%];\n        return switch_body(V, handlers);\n    };\n}\n", info_text);

        print(*builder, "variant_switch :: (v: $V/Variant%) {\n", parameter_text);
        print(*builder, "    copy := v;\n    variant_switch(*copy%);\n}\n", argument_text);
    }

    return builder_to_string(*builder,, temp);
}
