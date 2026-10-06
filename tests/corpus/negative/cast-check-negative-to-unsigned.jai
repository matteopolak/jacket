// A negative value cast to a narrower unsigned type does not fit it.
work :: () {
    n: s32 = -1;
    c := cast(u16) n;
}
#run work();
