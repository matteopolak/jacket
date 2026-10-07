// `#add_context name :: value;` declares a constant member of the Context (a module alias, as
// profiler plugins insert), reachable as `#Context.name` and `context.name`. It takes no space.
// rules: ctx.12
#import "Basic";

__Math :: #import "Math";

#add_context _Math :: __Math;
#add_context LIMIT :: 7;
#add_context counter := 2;

main :: () {
    assert(#Context ._Math.PI == __Math.PI);
    assert(context._Math.abs(-3) == 3);
    assert(#Context .LIMIT == 7);
    context.counter += #Context .LIMIT;
    assert(context.counter == 9);
    print("ok\n");
}
