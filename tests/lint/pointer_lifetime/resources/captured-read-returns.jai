#import "Basic";

Holder :: struct {
    value: *u8;
}

A :: struct {
    left: *u8;
    right: *u8;
}

B :: struct {
    pad: *u8;
    left: *u8;
}

make :: () -> *u8 {
    return cast(*u8) alloc(8);
}

read :: (p: *u8) -> u8 {
    return p.*;
}

get :: (p: *Holder) -> *u8 {
    return p.value;
}

get_b :: (p: *B) -> *u8 {
    return p.left;
}

take_old :: (p: *Holder, replacement: *u8) -> *u8 {
    old := p.value;
    p.value = replacement;
    return old;
}

pure_projection :: () {
    backing := make();
    holder: Holder;
    holder.value = backing;
    alias := get(*holder);

    free(backing);
    read(alias);
}

cast_other_field_stays_live :: () {
    left := make();
    right := make();
    holder: A;
    holder.left = left;
    holder.right = right;
    alias := get_b(cast(*B) *holder);

    free(left);
    read(alias);
}

captured_old_does_not_become_replacement :: () {
    old := make();
    replacement := make();
    holder: Holder;
    holder.value = old;
    alias := take_old(*holder, replacement);

    free(replacement);
    read(alias);
}

cast_same_field_dies :: () {
    left := make();
    right := make();
    holder: A;
    holder.left = left;
    holder.right = right;
    alias := get_b(cast(*B) *holder);

    free(right);
    read(alias);
}

main :: () {}
