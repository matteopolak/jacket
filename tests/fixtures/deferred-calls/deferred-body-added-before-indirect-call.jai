// Independent language/compiler reduction; no plugin names or library workaround.
#import "Basic";
C :: #import "Compiler";

#placeholder Supplied_Body;
BODY_SCOPE :: #code Supplied_Body;

compute :: () -> s64 {
    #insert Supplied_Body;
}

#run,stallable {
    C.add_build_string("Supplied_Body :: \"return 11;\";", -1, BODY_SCOPE);
    invoke: () -> s64 = compute;
    answer := invoke();
    if answer != 11 C.compiler_report("Deferred body returned the wrong value.");
}

main :: () {}
