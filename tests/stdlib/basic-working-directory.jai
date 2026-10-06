// get_working_directory / set_working_directory live in Basic (as in the reference), also with System and File imported.
#import "Basic";

main :: () -> s64 {
    original := copy_string(get_working_directory());
    if !original.count return 1;
    if !set_working_directory(original) return 2;
    if get_working_directory() != original return 3;
    if set_working_directory("/definitely/not/a/real/dir/xyz") return 4;
    sleep_milliseconds(1);
    print("ok\n");
    return 0;
}
