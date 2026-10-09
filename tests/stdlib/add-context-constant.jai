// `#add_context name :: value;` declares a constant member of the Context (a module alias, as
// profiler plugins insert), reachable as `#Context.name` and `context.name`. It takes no space.
// rules: ctx.12
#import "Basic";

__Math :: #import "Math";

#add_context _Math :: __Math;
#add_context LIMIT :: 7;
#add_context counter := 2;
#add_context _hook :: hook;

hook :: () -> int {
    return 42;
}

// The constants are listed in the Context's type info as CONSTANT members, procedures included,
// so compile-time code can detect a hook by name.
has_constant :: (name: string) -> bool {
    info := type_info(type_of(context));
    for info.members if it.name == name && (it.flags & .CONSTANT) return true;
    return false;
}

main :: () {
    assert(#Context ._Math.PI == __Math.PI);
    assert(context._Math.abs(-3) == 3);
    assert(#Context .LIMIT == 7);
    context.counter += #Context .LIMIT;
    assert(context.counter == 9);
    assert(context._hook() == 42);
    #if #run has_constant("_hook") {} else {
        assert(false);
    }
    #if #run has_constant("counter") {
        assert(false);
    }
    assert(has_constant("_hook") && has_constant("LIMIT"));
    assert(!has_constant("counter"));
    print("ok\n");
}
