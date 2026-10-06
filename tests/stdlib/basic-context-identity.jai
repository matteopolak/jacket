// Both annotations must resolve the selected canonical context nominal.
Basic :: #import "Basic";

copy_context :: (source: *Basic.Context) -> *#Context {
    return source;
}
copy_back :: (source: *#Context) -> *Basic.Context {
    return source;
}

main :: () -> s64 {
    selected: *#Context = *context;
    qualified: *Basic.Context = copy_back(selected);
    if copy_context(qualified) != selected return 1;
    return 0;
}
