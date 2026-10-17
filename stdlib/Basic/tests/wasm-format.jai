// No numerics imports: a canonical foundation/host ABI reproducer.
#import "Basic";

main :: () {
    text := sprint("%", cast(float64) 1.25);
    defer free(text);
    assert(text == "1.25");
    print("canonical float formatting ok\n");
}
