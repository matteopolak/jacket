// `xx` to a narrower type is checked like `cast`.
work :: () {
    n := 70000;
    c: s16 = xx n;
}
#run work();
