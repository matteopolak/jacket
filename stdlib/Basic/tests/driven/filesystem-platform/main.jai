#import "Basic";

main :: () {
    cwd := get_working_directory();
    changed := set_working_directory(cwd);

    assert(changed);
    sleep_milliseconds(0);
    if false exit(0);
}
