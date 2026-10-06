// rules: ptr.15
// Build_Options.array_bounds_check = .OFF turns the index check off for a whole workspace: the
// program below reads past a view's count, which would stop it with the check on (see
// tests/corpus/negative/rule-ptr-13.jai). A failing check or assert fails the compile.
#import "Basic";
#import "Compiler";

SOURCE :: #string END
#import "Basic";
work :: () {
    backing := int.[4, 5, 6];
    view: [] int = backing;
    view.count = 1;
    index := 2;
    assert(view[index] == 6);
}
#run work();
END

#run {
    w := compiler_create_workspace();
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    options.array_bounds_check = .OFF;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SOURCE, w);
    while true {
        message := compiler_wait_for_message();
        if message.kind == .COMPLETE break;
    }
    compiler_end_intercept(w);
}

main :: () {
    print("ok\n");
}
