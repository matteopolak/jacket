// A TYPECHECKED enum declaration carries its type in `Code_Enum.external_type`.
#import "Basic";
#import "Compiler";

PROGRAM :: #string END
Mode :: enum u8 { A; B; C; }
main :: () {}
END

#run {
    w := compiler_create_workspace("t");
    opts := get_build_options(w);
    opts.output_type = .NO_OUTPUT;
    set_build_options(opts, w);
    compiler_begin_intercept(w);
    add_build_string(PROGRAM, w);
    found := false;
    while true {
        msg := compiler_wait_for_message();
        if msg.kind == .TYPECHECKED {
            tc := cast(*Message_Typechecked) msg;
            for tc.declarations {
                e := it.expression.expression;
                if it.expression.name == "Mode" && e && e.kind == .ENUM {
                    info := cast(*Code_Enum) e;
                    assert(info.external_type != null && info.external_type.name == "Mode");
                    assert(info.external_type.values.count == 3);
                    found = true;
                }
            }
        }
        if msg.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
    assert(found, "Mode was not reported");
}

main :: () { print("ok\n"); }
