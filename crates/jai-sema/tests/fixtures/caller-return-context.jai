#add_context marker: int;
trace: int;
escape :: () #expand {
    defer trace = trace * 10 + context.marker;
    `defer trace = trace * 10 + context.marker;
    pushed: #Context;
    pushed.marker = 3;
    push_context pushed {
        defer trace = trace * 10 + context.marker;
        `return context.marker + 39;
    }
}
answer :: () -> int {
    context.marker = 1;
    pushed: #Context;
    pushed.marker = 2;
    push_context pushed {
        defer trace = trace * 10 + context.marker;
        escape();
    }
}
main :: () -> int {
    result := answer();
    if trace != 3222 return 1;
    return result;
}
