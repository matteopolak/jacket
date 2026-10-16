#import "Basic";
#load "file-private-import.jai";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok);
    free(path);
}
