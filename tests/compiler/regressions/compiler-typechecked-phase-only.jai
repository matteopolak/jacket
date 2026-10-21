// A metaprogram that only follows the phases still gets TYPECHECKED messages, and
// TYPECHECKED_ALL_WE_CAN, without reading what they hold (docs/metaprogramming/compiler-module.md).
// rules: compiler.15 records.4 ws.8
#import "Basic";
#import "Compiler";

TARGET :: #string JAI
#import "Basic";
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

    typechecked, settled := 0;

    while true {
        message := compiler_wait_for_message();
        if message.kind == {
            case .TYPECHECKED;
                typechecked += 1;
            case .PHASE;
                if (cast(*Message_Phase) message).phase == .TYPECHECKED_ALL_WE_CAN settled += 1;
            case .COMPLETE;
                assert((cast(*Message_Complete) message).error_code == .NONE);
                break;
        }
    }

    assert(typechecked > 0 && settled > 0);
    print("ok\n");
}
