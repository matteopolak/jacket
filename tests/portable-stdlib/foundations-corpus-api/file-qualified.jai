#import "Basic";
F :: #import "File";

main :: () {
    path, ok := F.get_absolute_path(relative_path = ".");
    assert(ok && path.count > 0 && path[0] == #char "/");
    free(path);
    print("ok\n");
}
