#import "Basic";
#import "Compiler";
#import "String";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok);
    free(path);
}
