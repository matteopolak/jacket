#import "Basic";

F :: #type (*[..] int);

factory :: () -> F {
    return selected_growth;
}

#scope_module
selected_growth :: (xs: *[..] int) {
    array_add(xs, 1);
}

unrelated :: () -> int {
    return 23;
}
