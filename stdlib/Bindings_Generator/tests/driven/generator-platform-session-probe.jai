// Feasibility probe only. No production activation protocol or public semantics.
#scope_file
#import "Basic";
B :: #import "Bindings_Generator";

Session :: struct {
    generator: *B.Generator;
    options: *B.Generate_Bindings_Options;
    cache: *[..] B.Library_Symbols;
}

#add_context _generator_platform_probe: *Session;

enter :: (session: *Session) -> *Session {
    assert(session != null && session.generator != null && session.options != null);
    previous := context._generator_platform_probe;
    context._generator_platform_probe = session;
    return previous;
}

leave :: (session: *Session, previous: *Session) {
    assert(context._generator_platform_probe == session);
    context._generator_platform_probe = previous;
}

nested_return :: (session: *Session) -> int {
    previous := enter(session);

    defer leave(session, previous);
    assert(context._generator_platform_probe.generator == session.generator);
    assert(context._generator_platform_probe.cache == session.cache);

    return 42;
}

#scope_export
main :: () {
    first_generator, second_generator: B.Generator;
    first_options, second_options: B.Generate_Bindings_Options;
    first_cache, second_cache: [..] B.Library_Symbols;
    first: Session = .{ generator = *first_generator, options = *first_options, cache = *first_cache };
    second: Session = .{ generator = *second_generator, options = *second_options, cache = *second_cache };
    library: B.Library_Info;

    array_add(*first_generator.libraries, *library);
    defer array_free(first_generator.libraries);
    assert(context._generator_platform_probe == null);
    previous := enter(*first);

    assert(previous == null);
    assert(nested_return(*second) == 42);
    assert(context._generator_platform_probe == *first);
    assert(context._generator_platform_probe.generator.libraries[0] == *library);
    assert(context._generator_platform_probe.options == *first_options);
    leave(*first, previous);
    assert(context._generator_platform_probe == null);

    push_context {
        context._generator_platform_probe = *second;
        assert(nested_return(*first) == 42);
        assert(context._generator_platform_probe == *second);
    }

    assert(context._generator_platform_probe == null);
    print("ok\n");
}
