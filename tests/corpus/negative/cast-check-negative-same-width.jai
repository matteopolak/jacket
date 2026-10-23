// A negative value does not fit an unsigned type of the same size.
work :: () {
    n := -1;
    c := cast(u64) n;
}
#run work();
