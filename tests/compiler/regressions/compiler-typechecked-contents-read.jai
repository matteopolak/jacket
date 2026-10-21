// A metaprogram that reads TYPECHECKED contents gets them in full
// (docs/metaprogramming/compiler-module.md).
// rules: compiler.15 records.4 records.8 ws.8
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
#import "Basic";
Point :: struct { x: int; y: int; }
square :: (x: int) -> int { return x * x; }
main :: () { print("%\n", square(7)); }
JAI

#run {
    w := compiler_create_workspace("target");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(TARGET, w);

    saw_point, saw_square, saw_main_body := false;

    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .TYPECHECKED;
                typechecked := cast(*Message_Typechecked) message;
                for typechecked.structs if it.expression.defined_type.name == "Point" saw_point = true;
                for typechecked.procedure_headers if it.expression.name == "square" saw_square = true;

                for typechecked.procedure_bodies {
                    assert(it.subexpressions.count > 0);
                    if it.expression.header.name == "main" saw_main_body = true;
                }
            case .COMPLETE;
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
        }
    }

    assert(saw_point && saw_square && saw_main_body);
    print("ok\n");
}
