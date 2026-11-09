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

main :: () {}
