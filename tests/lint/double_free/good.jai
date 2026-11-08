#import "Basic";

Node :: struct {
    value: int;
}

Holder :: struct {
    name: string;
}

release_holder :: (holder: *Holder) {
    free(holder.name);
}

// Frees the name and gives the field a new value, so a second call frees nothing.
reset_holder :: (holder: *Holder) {
    free(holder.name);
    holder.name = "";
}

// Frees only a copy of its own.
release_scratch :: (holder: *Holder) {
    scratch := copy_string("scratch");
    free(scratch);
}

// Frees only on one path.
release_sometimes :: (holder: *Holder, really: bool) {
    if really free(holder.name);
}

main_repeats :: (really: bool) {
    resettable := New(Holder);
    resettable.name = copy_string("held");
    reset_holder(resettable);
    reset_holder(resettable);

    scratch := New(Holder);
    release_scratch(scratch);
    release_scratch(scratch);

    sometimes := New(Holder);
    release_sometimes(sometimes, really);
    release_sometimes(sometimes, really);
}

main :: () {
    // A new value in between.
    node := New(Node);

    free(node);
    node = New(Node);
    free(node);

    // Different procedures: the contents first, then the memory holding them.
    holder := New(Holder);
    holder.name = copy_string("held");
    release_holder(holder);
    free(holder);

    // Only one of the two runs.
    text := copy_string("hello");
    if text.count > 3 {
        free(text);
    } else {
        free(text);
    }

    // The address may be used to change the variable.
    other := New(Node);
    slot := *other;

    free(other);
    slot.* = New(Node);
    free(other);

    // Left to the end of the scope.
    last := New(Node);

    defer free(last);
}
