#import "Basic";
#load "file-public-import.jai";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok && path.count > 0);
    free(path);
    print("ok\n");
}
