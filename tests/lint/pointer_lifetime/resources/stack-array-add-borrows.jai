#import "Basic";

shared_backing: [4] s64;

return_empty_slot :: () -> *s64 {
    backing: [4] s64;
    values: [..] s64;
    values.data = cast(*s64) *backing;
    values.allocated = 4;
    return array_add(*values);
}

return_last_slot :: () -> *s64 {
    backing: [4] s64;
    values: [..] s64;
    values.data = cast(*s64) *backing;
    values.allocated = 4;
    values.count = 3;
    return array_add(*values);
}

use_after_backing_scope :: () {
    stale: *s64;
    {
        backing: [4] s64;
        values: [..] s64;
        values.data = cast(*s64) *backing;
        values.allocated = 4;
        values.count = 1;
        stale = array_add(*values);
    }
    print("%\n", stale.*);
}

return_shared_slot :: () -> *s64 {
    values: [..] s64;
    values.data = cast(*s64) *shared_backing;
    values.allocated = 4;
    return array_add(*values);
}

keep_actual_backing :: () {
    live_backing: [4] s64;
    result: *s64;
    {
        old_backing: [4] s64;
        values: [..] s64;
        values.data = cast(*s64) *old_backing;
        values.allocated = 4;
        values.data = cast(*s64) *live_backing;
        result = array_add(*values);
    }
    result.* = 7;
    print("%\n", result.*);
}

main :: () {}
