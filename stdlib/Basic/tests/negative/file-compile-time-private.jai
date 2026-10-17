#import "Basic";
#import "String";
#load "file-private-import.jai";

#run {
    path, ok := get_absolute_path(".");
    assert(ok);
    free(path);
}

main :: () {}
