#import "Basic";
R :: #import "rpmalloc"(USE_RAW = true);

main :: () {
    assert(R.rpmalloc_initialize() == 0);
    p := R.rpmalloc_allocator_proc(.ALLOCATE, 4096, 0, null, null);

    assert(p != null);
    R.rpmalloc_allocator_proc(.FREE, 0, 0, p, null);
    R.rpmalloc_finalize();
    print("ok\n");
}
