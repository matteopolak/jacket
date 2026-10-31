#import "Basic";

main :: () {
    original := get_working_directory();

    defer free(original);
    assert(original.count > 0);
    assert(set_working_directory("."));
    current := get_working_directory();

    defer free(current);
    assert(current == original);
    assert(!set_working_directory("/jai-foundations-missing-directory/no-such-path"));
    print("ok\n");
    exit(0);
}
