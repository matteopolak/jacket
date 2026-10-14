#import "Basic";
M :: #import "Record";

main :: () {
    value := M.create();
    assert(value != null && value.value == 0);
    free(value);
}
