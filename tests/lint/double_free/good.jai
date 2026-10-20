#import "Basic";

Node :: struct {
    value: int;
}

Holder :: struct {
    name: string;
}

release_holder :: (holder: *Holder) {
    free(holder.name);
} @FreesArgument

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
