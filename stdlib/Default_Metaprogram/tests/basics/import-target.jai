using #import "Basic";
Dependency :: #import "Default_Target_Dependency";

main :: () {
    assert(Dependency.VALUE == 42);
    print("import ok\n");
}
