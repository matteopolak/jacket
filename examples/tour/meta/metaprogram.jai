// The compiler is a library you can drive from your own code, at compile time.
//
// The #run below starts a second compilation in a fresh workspace, feeds it a
// small program as a string, and listens to the compiler's messages as it
// works. A real build script does the same to add files, check style rules,
// or generate code from the program it is compiling.

#import "Compiler";

PATIENT :: #string DONE
// A tiny program for the metaprogram to inspect.
Point :: struct { x, y: int; }
add :: (a: Point, b: Point) -> Point { return .{ a.x + b.x, a.y + b.y }; }
lengthSquared :: (p: Point) -> int { return p.x * p.x + p.y * p.y; } // camelCase!
origin :: () -> Point { return .{}; } @Todo
main :: () { p := add(origin(), .{ 3, 4 }); }
DONE

INSPECTION :: #run inspect(PATIENT);

inspect :: (source: string) -> string {
    workspace := compiler_create_workspace("inspection");
    options := get_build_options(workspace);
    options.output_type = .NO_OUTPUT; // type-check only, don't produce a program
    set_build_options(options, workspace);

    compiler_begin_intercept(workspace);
    add_build_string(source, workspace);

    report: String_Builder;
    messages := 0;
    while true {
        message := compiler_wait_for_message();
        messages += 1;
        if message.kind == .COMPLETE break;
        if message.kind != .TYPECHECKED continue;

        typechecked := cast(*Message_Typechecked) message;
        for typechecked.procedure_headers {
            header := it.expression;
            if !from_main_program(header) continue;
            print_to_builder(*report, "  procedure %: % argument(s), % return value(s)\n",
                header.name, header.arguments.count, header.returns.count);
            if !is_snake_case(header.name) {
                print_to_builder(*report, "    style: '%' should be snake_case\n", header.name);
            }
        }
        for typechecked.structs {
            info := it.expression.defined_type;
            if info && info.name == "Point" {
                print_to_builder(*report, "  struct %: % members\n", info.name, info.members.count);
            }
        }
        for typechecked.declarations {
            declaration := it.expression;
            for declaration.notes {
                if it.text == "Todo" {
                    print_to_builder(*report, "    note: '%' is marked @Todo\n", declaration.name);
                }
            }
        }
    }
    compiler_end_intercept(workspace);
    print_to_builder(*report, "  (the compiler sent % messages)\n", messages);
    return builder_to_string(*report);
}

from_main_program :: (node: *Code_Node) -> bool {
    if !node.enclosing_load return false;
    imported := node.enclosing_load.enclosing_import;
    return imported && imported.module_type == .MAIN_PROGRAM;
}

is_snake_case :: (name: string) -> bool {
    for name if it >= #char "A" && it <= #char "Z" return false;
    return true;
}

tour_metaprogram :: () {
    print("While this tour was compiling, it compiled another program and took notes:\n");
    print("%", INSPECTION);
}
