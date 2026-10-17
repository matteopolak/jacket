#import "Basic";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok);
    free(path);
}
