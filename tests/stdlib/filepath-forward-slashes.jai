// `#file` and `#filepath` use `/` on every platform, so a metaprogram can splice them into a
// string literal (`#load "<dir>/module.jai"`) without escaping Windows backslashes.
// rules: proc.7
#import "Basic";

main :: () {
    for 0..#filepath.count - 1 assert(#filepath[it] != #char "\\", "#filepath has a backslash: %", #filepath);
    for 0..#file.count - 1 assert(#file[it] != #char "\\", "#file has a backslash: %", #file);
    assert(#filepath.count > 0 && #filepath[#filepath.count - 1] == #char "/");
    print("ok\n");
}
