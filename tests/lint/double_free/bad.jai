#import "Basic";

Node :: struct {
    value: int;
}

main :: () {
    node := New(Node);

    free(node);
    free(node);

    text := copy_string("hello");

    print("%\n", text);
    free(text);
    free(text);

    items := array_copy(int.[1, 2, 3]);

    array_free(items);
    array_free(items);
}

Holder :: struct {
    name: string;
    items: [..] int;
}

release_name :: (holder: *Holder) {
    free(holder.name);
}

// Reaches the storage through another helper.
release_all :: (holder: *Holder) {
    release_name(holder);
    array_reset(*holder.items);
}

release_twice :: () {
    holder := New(Holder);
    holder.name = copy_string("held");

    release_name(holder);
    release_name(holder);
}

release_all_twice :: () {
    holder := New(Holder);
    holder.name = copy_string("held");

    release_all(holder);
    release_all(holder);
}
