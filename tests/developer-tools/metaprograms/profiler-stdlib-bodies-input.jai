// Own minimal real-library input for the copied producer driver.
#import "Basic";

main :: () {
    memory := alloc(16);

    assert(memory != null);
    p := cast(*u8) memory;
    p[0] = 42;
    assert(p[0] == 42);
    free(memory);
}
