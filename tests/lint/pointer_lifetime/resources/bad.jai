#import "Basic";
#load "macros.jai";

escaped: *int;
saved_any: Any;

release_inferred :: (p: *void) {
    free(p);
}

close_handle :: (handle: int) {} @Closes

alias_after_free :: () {
    p := New(int);
    q := p;
    release_inferred(p);
    print("%\n", q.*);
}

free_cut :: (flag: bool) {
    p := New(int);
    if flag free(p);
    else free(p);
    print("%\n", p.*);
}

deferred_result :: () -> *int {
    p := New(int);
    defer free(p);
    return p;
}

temporary_reset :: () {
    text := tprint("temporary");
    reset_temporary_storage();
    print("%\n", text);
}

temporary_restore :: () {
    mark := get_temporary_storage_mark();
    text := tprint("after mark");
    set_temporary_storage_mark(mark);
    print("%\n", text);
}

temporary_deferred :: () -> string {
    auto_release_temp();
    return tprint("released by defer");
} @TempAllocates

handle_use :: () {
    handle := 7;
    close_handle(handle);
    print("%\n", handle);
}

handle_double_close :: () {
    handle := 7;
    close_handle(handle);
    close_handle(handle);
}

alias_double_free :: () {
    p := New(int);
    q := p;
    free(p);
    free(q);
}

block_expired :: () {
    p: *int;

    {
        value := 4;
        p = *value;
    }

    print("%\n", p.*);
}

iterator_copy :: () {
    values := int.[1, 2];
    for values escaped = *it;
}

macro_pointer :: () -> *int #expand {
    value := 3;
    return *value;
}

expanded_local :: () -> *int {
    return macro_pointer();
}

boxed_local :: () {
    value := 3;
    saved_any = value;
}

boxed_temporary :: () {
    local: Any = get_number();
    print("%\n", local);
}

get_number :: () -> int {
    return 3;
}

by_value_parameter :: (value: int) -> *int {
    return *value;
}

return_temporary :: () -> string {
    text := tprint("forwarded");
    alias := text;
    return alias;
}

main :: () {}

Holder :: struct {
    value: *int;
}

return_pointer_field :: () -> Holder {
    local := 3;
    holder: Holder;
    holder.value = *local;
    return holder;
}

Box :: struct {
    value: Any;
}

return_boxed_field :: () -> Box {
    local := 3;
    holder: Box;
    holder.value = local;
    return holder;
}

release_field :: (holder: *Holder) {
    free(holder.value);
}

alias_after_field_release :: () {
    holder: Holder;
    holder.value = New(int);
    alias := holder.value;

    release_field(*holder);
    print("%\n", alias.*);
}

boxed_parameter :: (value: int) -> Any {
    return value;
}

field_double_free :: () {
    holder: Holder;
    holder.value = New(int);
    release_field(*holder);
    release_field(*holder);
}

imported_expansion :: () {
    pointer := New(int);
    free_and_read(pointer);
}

escaped_temporary: string;

retain_returned_temporary :: () {
    escaped_temporary = return_temporary();
}
