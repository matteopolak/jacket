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
