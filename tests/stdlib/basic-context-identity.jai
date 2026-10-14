// The runtime context is named by the language's #Context type.
copy_context :: (source: *#Context) -> *#Context {
    return source;
}

main :: () -> s64 {
    selected: *#Context = *context;
    if copy_context(selected) != selected return 1;
    return 0;
}
