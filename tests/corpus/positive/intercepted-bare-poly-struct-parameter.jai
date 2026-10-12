// A procedure taking a polymorphic struct named bare (`f: *F`) stays polymorphic when the
// workspace's typechecked declarations are exported for an intercept before anything calls
// it: here a local named like it (`inner: bool`) is reached first.
#import "Compiler";

SRC :: #string DONE
F :: struct (P: Type = int) {}
outer :: (f: *F) {
    inner: bool;
    y := inner && true;
}
rc :: (f: *F) {}
inner :: (f: *F) { rc(f); }
main :: () { f: F(int); outer(*f); }
DONE

#run {
    w := compiler_create_workspace("child");
    options := get_build_options(w);
    options.output_type = .NO_OUTPUT;
    set_build_options(options, w);
    compiler_begin_intercept(w);
    add_build_string(SRC, w);
    compiler_end_intercept(w);
}

main :: () -> int {
    return 17;
}
