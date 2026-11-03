#import "Basic";

Item :: struct {
    text: string;
}

shared: Item;

fill :: (destination: *Item) {
    destination.text = tprint("%", 1);
}

safe_local :: () {
    item: Item;
    fill(*item);
    assert(item.text == "1");
}

bad_return :: () -> string {
    item: Item;
    fill(*item);
    return item.text;
}

bad_global :: () {
    fill(*shared);
}

bad_heap :: () -> *Item {
    item := New(Item);
    fill(item);
    return item;
}

shared_pointer: *void;

fill_void :: (destination: *void) {
    (cast(**void) destination).* = talloc(8);
}

safe_void_local :: () {
    local: *void;
    fill_void(*local);
    assert(local != null);
}

bad_void_return :: () -> *void {
    local: *void;
    fill_void(*local);
    return local;
}

bad_void_global :: () {
    fill_void(*shared_pointer);
}

safe_temporary_receiver :: () {
    memory := talloc(size_of(*void));
    fill_void(memory);
}

append_conditional :: (destination: *[..] string, convert: bool) {
    text := "literal";

    if convert {
        owned := copy_string("seven");
        text = copy_temporary_string(owned);
        free(owned);
    }

    array_add(destination, text);
}

safe_conditional_local :: (convert: bool) {
    values: [..] string;
    values.allocator = temp;
    append_conditional(*values, convert);
    assert(values.count == 1);
}

shared_values: [..] string;

bad_conditional_global :: (convert: bool) {
    append_conditional(*shared_values, convert);
}

// The stack slot and the temporary allocation it holds have different lifetimes.
fill_stack_address :: (destination: ***void) {
    local := talloc(8);
    destination.* = *local;
}

shared_slot: **void;

bad_stack_address :: () {
    fill_stack_address(*shared_slot);
}

Pair :: struct {
    temporary: *void;
    owned: *void;
}

fill_pair :: (destination: *Pair, convert: bool) {
    data: *void;
    if convert data = talloc(8);
    destination.* = .{ temporary = data, owned = alloc(8) };
}

safe_owned_field :: (convert: bool) -> *void {
    pair: Pair;
    fill_pair(*pair, convert);
    return pair.owned;
}

bad_temporary_field :: (convert: bool) -> *void {
    pair: Pair;
    fill_pair(*pair, convert);
    return pair.temporary;
}

shared_pair: Pair;

bad_global_pair :: (convert: bool) {
    fill_pair(*shared_pair, convert);
}

fill_string_fields :: (destination: *string, convert: bool) {
    text: string;

    if convert {
        text.data = cast(*u8) talloc(8);
        text.count = 8;
    }

    destination.* = text;
}

safe_string_fields :: (convert: bool) {
    text: string;
    fill_string_fields(*text, convert);
    assert(text.count == 0 || text.count == 8);
}

shared_text: string;

bad_string_fields :: (convert: bool) {
    fill_string_fields(*shared_text, convert);
}
