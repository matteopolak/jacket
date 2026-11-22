main :: () {
    heap := make_heap();

    push_allocator(heap_allocator(*heap));

    array: [..] int;
    for 1..200 array_add(*array, it);

    total := 0;
    for array total += it;

    print("% %\n", array.count, total);

    names: [..] string;

    array_add(*names, "a");
    array_add(*names, tprint("%-%", 1, 2));
    print("%\n", names[1]);
}
