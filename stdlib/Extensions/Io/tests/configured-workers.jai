#import "Basic";
Io :: #import "Extensions/Io";
A :: #import "Atomics";

completed: s32;

worker :: () {
    Io.sleep(10);
    for 1..10 Io.yield();
    A.atomic_add(*completed, 1);
}

main :: () {
    Io.io_run_threads(4, worker);

    // Verification remains mandatory when the child MAIN disables Basic.assert.
    if A.atomic_read(*completed) != 4 {
        print("ERROR: not all configured Io workers completed\n");
        exit(1);
    }

    assert(A.atomic_read(*completed) == 4);
    print("ok\n");
}
