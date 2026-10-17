#import "Basic";
#load "named-private-file.jai";

main :: () {
    path, ok := Hidden_File.get_absolute_path(".");
    assert(ok);
    free(path);
}
