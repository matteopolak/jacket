#import "Basic";

Parser :: struct {
    depth: int;
}

registry: [..] *Parser;

consume :: (parser: *Parser) {
    print("depth %\n", parser.depth);
}

make :: () -> *Parser {
    parser := New(Parser);
    parser.depth = 1;
    return parser;
}

// Temporary storage is reset by the program, not freed.
temporary :: () {
    push_allocator(temp);

    scratch := New(Parser);
    scratch.depth = 4;
}

temporary_call :: () {
    quick := New(Parser,, allocator = temp);
    quick.depth = 5;
}

main :: () {
    // Freed, directly and by `defer`.
    freed := New(Parser);
    freed.depth = 1;
    free(freed);

    deferred := New(Parser);

    defer free(deferred);
    deferred.depth = 2;

    // Handed to another procedure, or kept somewhere that outlives the local.
    passed := New(Parser);

    consume(passed);

    kept := New(Parser);

    array_add(*registry, kept);

    // The address of the local may be handed on.
    owner := New(Parser);
    slot := *owner;
    slot.* = null;

    other := make();

    consume(other);
    free(other);

    temporary();
    temporary_call();
}
