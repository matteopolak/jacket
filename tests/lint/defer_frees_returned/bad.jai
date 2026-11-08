#import "Basic";
#import "String";

make_name :: (first: string) -> string {
    name := join(first, " Smith");
    defer free(name);
    return name;
}

make_buffer :: (size: int, zeroed: bool) -> *u8 {
    buffer := cast(*u8) alloc(size);

    defer free(buffer);

    if zeroed {
        memset(buffer, 0, size);
        return buffer;
    }

    return xx buffer;
}

Registry :: struct {
    label: string;
}

// Frees what the registry owns.
clear_registry :: (registry: *Registry) {
    free(registry.label);
}

make_registry :: () -> *Registry {
    registry := New(Registry);
    registry.label = copy_string("kept");

    defer clear_registry(registry);
    return registry;
}

main :: () {
    print("%\n", make_registry());
    print("%\n", make_name("Ann"));
    print("%\n", make_buffer(8, true));
}
