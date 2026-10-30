// test: compile_fail
M :: #import "no-parameter-module"(EXTRA = true);

main :: () {
    result := M.touch();
}
