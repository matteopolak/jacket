#import "Basic";

main :: () {
    buffer := alloc(64);
    names := string.["a", "b", "c"];

    for names {
        defer free(buffer);
        print("%\n", it);
    }

    items: [..] int;
    i := 0;

    while i < 2 {
        i += 1;
        defer array_reset(*items);
    }
}

dispose :: (memory: *void) {
    free(memory);
} @FreesArgument

noted_release :: () {
    buffer := alloc(8);

    for 1..2 {
        defer dispose(buffer);
    }
}

dispose_quietly :: (memory: *void) {
    free(memory);
}

unnoted_release :: () {
    buffer := alloc(8);

    for 1..2 {
        defer dispose_quietly(buffer);
    }
}

Registry :: struct {
    label: string;
    items: [..] int;
}

// Frees what the registry owns, then makes it reusable.
clear_registry :: (registry: *Registry) {
    free(registry.label);
    array_reset(*registry.items);
    registry.* = .{};
}

clear_in_loop :: () {
    registry: Registry;

    for 1..2 {
        defer clear_registry(*registry);
    }
}
