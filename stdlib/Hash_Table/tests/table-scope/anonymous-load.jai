#import "Basic";
#load "anonymous-file.jai";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok && path.count > 0);
    free(path);
    print("ok\n");
}
