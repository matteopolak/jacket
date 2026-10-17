#import "Basic";
F :: #import "File";

#run {
    path, ok := F.get_absolute_path(".");
    assert(ok);
    free(path);
}

main :: () {
    print("ok\n");
}
