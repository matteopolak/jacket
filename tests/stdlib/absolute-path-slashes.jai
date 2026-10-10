// Absolute paths use `/` on Windows too, so a metaprogram can put one into a string literal or a
// `#load` (stubborn builds `#load "<absolute dir>/tests/x.spec.jai"` that way).
#import "Basic";
#import "File";

main :: () {
    path, ok := get_absolute_path(".");
    assert(ok && path.count > 0, "no absolute path");
    for *path assert(it.* != #char "\\", "backslash in '%'", path);
    print("ok\n");
}
