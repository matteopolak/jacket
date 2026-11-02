#import "Basic";

Point :: struct {
    value: int;
}

close_handle :: (handle: int) {} @Closes

conditional_free :: (flag: bool) {
    p := New(int);
    if flag free(p);
    print("%\n", p.*);
}

replaced :: () {
    p := New(int);

    free(p);
    p = New(int);
    print("%\n", p.*);
    free(p);
}

deferred_last :: () {
    p := New(int);
    defer free(p);
    print("%\n", p.*);
}

local_scope :: () {
    value := 4;
    p := *value;
    print("%\n", p.*);
}

before_watermark :: () {
    text := tprint("before mark");
    mark := get_temporary_storage_mark();

    tprint("discarded");
    set_temporary_storage_mark(mark);
    print("%\n", text);
}

reset_and_replace :: () {
    text := tprint("old");
    reset_temporary_storage();
    text = tprint("new");
    print("%\n", text);
}

owned_copy :: () -> string {
    auto_release_temp();
    return copy_string(tprint("copied"));
}

borrowed_parameter :: (p: *Point) -> *int {
    return *p.value;
}

boxed_for_call :: () {
    local := 3;
    print("% %\n", local, get_number());
}

get_number :: () -> int {
    return 3;
}

replaced_handle :: () {
    handle := 7;
    close_handle(handle);
    handle = 8;
    close_handle(handle);
}

static_text := tprint("baked into static data");

// Registration can be conditional: only strict mode may report this path.
conditional_cleanup :: (flag: bool) -> string {
    if flag auto_release_temp();
    return tprint("possibly released");
} @TempAllocates

main :: () {}

cleared: *int;

escape_cleared_before_exit :: () {
    local := 3;
    cleared = *local;
    print("%\n", cleared.*);
    cleared = null;
}

Holder :: struct {
    value: *int;
}

rebind_then_release :: (holder: *Holder) {
    holder.value = New(int);
    free(holder.value);
}

old_allocation_survives_rebinding :: () {
    holder: Holder;
    holder.value = New(int);
    alias := holder.value;

    rebind_then_release(*holder);
    print("%\n", alias.*);
    free(alias);
}

forward_box :: (value: Any) -> Any {
    return value;
}
