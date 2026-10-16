#import "Basic";
#load "named-private-file.jai";

main :: () {
    path, ok := File.get_absolute_path(".");
    assert(ok);
    free(path);
}
