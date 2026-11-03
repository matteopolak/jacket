#import "Basic";

safe_scoped_context :: () {
    local := 1;

    push_context {
        context.logger_data = *local;
        assert((cast(*int) context.logger_data).* == 1);
    }
}

bad_caller_context :: () {
    local := 1;
    context.logger_data = *local;
}

retired: [..] *int;

safe_cleared_global_container :: () {
    local := 1;
    array_add(*retired, *local);
    array_free(retired);
    retired = .{};
}

bad_retained_global_container :: () {
    local := 1;
    array_add(*retired, *local);
}

A :: #import "global-holders";
B :: #import "global-holders";

safe_qualified_global_container :: () {
    local := 1;

    array_add(*A.retired, *local);
    assert(B.retired.count == 1);
    array_free(A.retired);
    A.retired = .{};
}

bad_qualified_global_container :: () {
    local := 1;
    array_add(*A.retired, *local);
}
