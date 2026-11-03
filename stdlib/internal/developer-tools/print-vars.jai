append_indented :: (builder: *String_Builder, field: string, indentation: Print_Vars_Config.Indentation, gap: s64) {
    if indentation == .RIGHT_JUSTIFIED for 0..gap - 1 append(builder, " ");
    append(builder, field);
    if indentation == .LEFT_JUSTIFIED for 0..gap - 1 append(builder, " ");
}

print_helper :: (builder: *String_Builder, using config: Print_Vars_Config, name_width: s64, value_width: s64, name: string, value: string, type: Type) {
    if control_flags & .PRINT_NAME {
        append(builder, before_name);
        append_indented(builder, name, name_indentation, max(cast(s64) 0, name_width - name.count));
        append(builder, after_name);
    }

    if control_flags & .PRINT_VALUE {
        append(builder, before_value);
        append_indented(builder, value, value_indentation, max(cast(s64) 0, value_width - value.count));
        append(builder, after_value);
    }

    if control_flags & .PRINT_TYPE {
        append(builder, before_type);
        print_to_builder(builder, "%", type);
        append(builder, after_type);
    }

    append(builder, line_separator);
}

// The source text of `code`, rendered at compile time.
get_code_name :: (code: Code) -> string #compile_time {
    root := compiler_get_nodes(code);
    builder: String_Builder;

    defer free_buffers(*builder);

    if !PP.print_expression(*builder, root) {
        compiler_report("Print_Vars: cannot render the caller expression");
        return "";
    }

    return builder_to_string(*builder);
}

print_vars_helper :: (builder: *String_Builder, config: Print_Vars_Config, $args: [$N] Code, loc: Source_Code_Location) #expand {
    assert(builder != null, "Print_Vars requires a builder", loc = loc);
    names: [N] string;
    values: [N] string;
    types: [N] Type;

    capture_vars(args, 0, *names, *values, *types);
    name_width: s64;
    value_width: s64;

    for names name_width = max(name_width, it.count);
    for values value_width = max(value_width, it.count);
    for index: 0..N - 1 print_helper(builder, config, name_width, value_width, names[index], values[index], types[index]);
}

// Appends one line per expression to `builder`, each with the expression's source text, its value
// and its type, lined up in columns.
//
// ```jai
// x: s32 = 7;
// y: s32 = 22;
// builder: String_Builder;
//
// defer free_buffers(*builder);
// print_vars_to(*builder, x, y);
// text := builder_to_string(*builder);
//
// defer free(text);
// assert(text == "x = 7  (s32)\ny = 22 (s32)\n");
// ```
print_vars_to :: (builder: *String_Builder, $args: ..Code, loc := #caller_location) #expand {
    print_vars_helper(builder, Print_Vars_Config.{}, #run fixed_codes(args.count, args), loc);
}

// Prints each expression's source text, value and type, one line apiece. Each expression is
// evaluated once.
//
// ```jai
// width := 640;
// print_vars(width, width * 2);
// // width       = 640  (s64)
// // (width * 2) = 1280 (s64)
// ```
print_vars :: ($args: ..Code, loc := #caller_location) #expand {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_vars_helper(*builder, Print_Vars_Config.{}, #run fixed_codes(args.count, args), loc);
    text := builder_to_string(*builder);

    defer free(text);
    print("%", text);
}

// Like `print_vars`, but sends the lines through `log`.
log_vars :: ($args: ..Code, loc := #caller_location) #expand {
    builder: String_Builder;

    defer free_buffers(*builder);
    print_vars_helper(*builder, Print_Vars_Config.{}, #run fixed_codes(args.count, args), loc);
    text := builder_to_string(*builder);

    defer free(text);
    log("%", text, loc = loc);
}

#scope_file
#import "Basic";
#import "Compiler";
PP :: #import "Program_Print";

fixed_codes :: ($count: s64, args: [] Code) -> [count] Code {
    result: [count] Code;
    for index: 0..args.count - 1 result[index] = args[index];
    return result;
}

capture_vars :: ($args: [$N] Code, $index: s64, names: *[N] string, values: *[N] string, types: *[N] Type) #expand {
    #if index < N {
        value := #insert args[index];

        names.*[index] = #run get_code_name(args[index]);
        values.*[index] = tprint("%", value);
        types.*[index] = type_of(value);
        capture_vars(args, index + 1, names, values, types);
    }
}
