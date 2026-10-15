// Original general linter-host dependency: real SDK executable-path query in #run.
S :: #import "System";

PATH :: #run S.get_path_of_running_executable();

using #import "Basic";

main :: () {
    assert(PATH.count > 0);
    print("ok\n");
}
