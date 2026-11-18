main :: () {
    f(1, 2,,, allocator = temp);
    f(1, 2 , ,, allocator = temp);
    f(1, 2,, allocator = temp);
    f(,, allocator = temp);
}
