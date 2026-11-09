#import "Basic";

Holder :: struct {
    storage: *int;
}

replace_then_free :: (holder: *Holder, replacement: *int) {
    holder.storage = replacement;
    free(holder.storage);
}

free_then_replace :: (holder: *Holder, replacement: *int) {
    free(holder.storage);
    holder.storage = replacement;
}

unsafe_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    replace_then_free(*holder, replacement);
    print("%", replacement.*); // replacement was freed
    free(initial);
}

safe_initial :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    replace_then_free(*holder, replacement);
    print("%", initial.*);
    free(initial);
}

safe_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    free_then_replace(*holder, replacement);
    print("%", replacement.*);
    free(replacement);
}

// The field holds the replacement after the call, so reading it is fine.
refilled_field :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    free_then_replace(*holder, replacement);
    print("%", holder.storage.*);
    free(replacement);
}

// The store and the free sit in different blocks; the field holds the replacement on every path.
replace_then_free_after_branch :: (holder: *Holder, replacement: *int, loud: bool) -> int {
    holder.storage = replacement;
    noise := 0;
    if loud noise += 1;

    free(holder.storage);

    return noise;
}

replace_then_free_in_loop :: (holder: *Holder, replacement: *int, count: int) -> int {
    holder.storage = replacement;
    total := 0;
    for 1..count total += it;

    free(holder.storage);

    return total;
}

// Only one path stores the replacement, so the free releases either value.
replace_on_one_path :: (holder: *Holder, replacement: *int, replace: bool) {
    if replace holder.storage = replacement;
    free(holder.storage);
}

unsafe_branched_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    _ = replace_then_free_after_branch(*holder, replacement, true);
    print("%", replacement.*); // replacement was freed
    free(initial);
}

unsafe_looped_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    _ = replace_then_free_in_loop(*holder, replacement, 3);
    print("%", replacement.*); // replacement was freed
    free(initial);
}

safe_branched_initial :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    _ = replace_then_free_after_branch(*holder, replacement, true);
    print("%", initial.*);
    free(initial);
}

safe_one_path_replacement :: () {
    initial := New(int);
    replacement := New(int);
    holder := Holder.{ initial };

    replace_on_one_path(*holder, replacement, true);
    print("%", replacement.*);
    free(replacement);
}

main :: () {}
