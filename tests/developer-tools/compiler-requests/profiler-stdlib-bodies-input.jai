// Own minimal real-library input for the copied producer driver.
#import "Basic";

main :: () {
    memory := alloc(16);
    assert(memory != null);
    free(memory);
}
