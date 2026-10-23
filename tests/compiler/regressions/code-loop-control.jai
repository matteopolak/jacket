// `#code continue` (and `break`, `remove`) is a loop-control node, so a metaprogram can splice it
// into generated statements.
#import "Basic";
#import "Compiler";

skip :: #code continue;

main :: () {
    kind :: #run -> Code_Node.Kind {
        return compiler_get_nodes(#code continue).kind;
    };

    assert(kind == .LOOP_CONTROL, "got %", kind);
    assert(#run compiler_get_nodes(#code break).kind == .LOOP_CONTROL);

    total := 0;

    for 1..4 {
        if it == 2 #insert skip;
        total += it;
    }

    assert(total == 8, "got %", total);
    print("ok\n");
}
