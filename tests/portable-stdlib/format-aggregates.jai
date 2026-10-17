#import "Basic";

Payload :: struct {
    number: int;
    text: string;
    CONSTANT :: 55;
}

Nested :: struct {
    value: FormatInt;
}

Plain :: enum {
    ZERO :: 0;
    ONE :: 1;
}

Flags :: enum_flags u32 {
    NONE :: 0;
    A :: 1;
    B :: 2;
}

Meters :: #type,isa s64;

main :: () {
    value := Payload.{ 7, "a\nb" };

    assert(sprint("%", value) == "{7, \"a\\nb\"}");
    assert(sprint("%", formatStruct(value, use_long_form_if_more_than_this_many_members = 1)) == "{number = 7; text = \"a\\nb\"; }");
    assert(sprint("%", formatStruct(*value)) == "{7, \"a\\nb\"}");
    multiline := formatStruct(value, use_long_form_if_more_than_this_many_members = -1, use_newlines_if_long_form = true);
    multiline.draw_type_name = true;
    multiline.indentation_width = 2;
    assert(sprint("%", multiline) == "Payload{\n  number = 7; \n  text = \"a\\nb\"; \n}");
    assert(context.print_style.indentation_depth == 0 && context.print_style.state_flags == 0);
    values: [4] int = .[1, 2, 3, 4];

    assert(sprint("%", values) == "[1, 2, 3, 4]");
    view: [] int = values;

    assert(sprint("%", view) == "[1, 2, 3, 4]");
    empty_view: [] int;
    empty_fixed: [0] int;

    assert(sprint("% %", empty_view, empty_fixed) == "[] []");
    many: [101] int;
    for *element: many element.* = 42;

    limited := sprint("%", many);

    defer free(limited);
    assert(limited.count == 403 && limited[397] == 52 && limited[398] == 50);
    assert(limited[399] == 46 && limited[400] == 46 && limited[401] == 46 && limited[402] == 93);
    formatted: FormatArray;
    formatted.value = view;
    formatted.stop_printing_after_this_many_elements = 2;
    assert(sprint("%", formatted) == "[1, 2...]");
    formatted.draw_separator_after_last_element = true;
    assert(sprint("%", formatted) == "[1, 2, ...]");
    formatted.stop_printing_after_this_many_elements = -1;
    formatted.draw_separator_after_last_element = true;
    assert(sprint("%", formatted) == "[1, 2, 3, 4, ]");
    formatted.stop_printing_after_this_many_elements = 0;
    assert(sprint("%", formatted) == "[...]");
    assert(sprint("% % %", Plain.ONE, Flags.NONE, Flags.A | Flags.B) == "ONE NONE A | B");
    assert(sprint("%", cast(Flags) 9) == "A | 0x8");
    assert(sprint("%", cast(Meters) 12) == "12");
    null_pointer: *Payload;

    assert(sprint("%", null_pointer) == "null");
    assert(sprint("%", cast(*void) 0x12345678) == "1234_5678");
    empty: Any;

    assert(sprint("%", empty) == "Any.{}");
    number: int = 17;
    borrowed := Any.{ type = type_info(int), value_pointer = *number };

    assert(sprint("%", borrowed) == "17");
    number = 19;
    assert(sprint("%", borrowed) == "19");
    // Partially null Any is a required fault, covered separately by
    // format-any-missing-value.jai and format-any-missing-type.jai.
    none: FormatInt;

    assert(sprint("%", none) == "(null Any)");
    assert(sprint("% % %", FormatFloat.{}, FormatStruct.{}, FormatArray.{}) == "(null Any) (null Any) (null Any)");
    errors: int;

    push_context {
        context.logger = count_error;
        context.logger_data = *errors;
        assert(sprint("%", formatStruct(7)) == "error");
        assert(errors == 1);
        context.print_style.log_runtime_errors = false;
        assert(sprint("%", formatStruct(7)) == "error");
        assert(errors == 1);
    }

    nested := Nested.{ formatInt(15, base = 16) };
    text := sprint("%", nested);

    defer free(text);
    assert(text != "{f}");
    member := isFormatter(cast(*Type_Info_Struct) type_info(FormatInt));

    assert(member != null && member.type == type_info(Formatter));

    push_context {
        context.print_style.struct_printer = custom;
        assert(sprint("%", value) == "custom");
        assert(sprint("%", *value) == "pointer");
        assert(sprint("%", cast(Meters) 12) == "meters");
    }

    print("ok\n");
}

custom :: (builder: *String_Builder, any: Any, struct_printer_data: *void) -> bool {
    if any.type == type_info(*Payload) {
        append(builder, "pointer");
        return true;
    }

    if any.type == type_info(Meters) {
        append(builder, "meters");
        return true;
    }

    if any.type != type_info(Payload) return false;

    append(builder, "custom");

    return true;
}

count_error :: (message: string, data: *void, info: Log_Info) {
    assert(message.count > 0 && (info.common_flags & .ERROR) != 0);
    (cast(*int) data).* += 1;
}
