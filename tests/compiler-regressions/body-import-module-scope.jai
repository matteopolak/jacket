// An `#import` written inside a procedure body joins the module's imports, like a top-level
// anonymous one: another file of the same module can use its names.
#import "Basic";
Scope :: #import "Body_Import_Scope";

main :: () {
    if Scope.root_of_sixteen() != 4 exit(1);
    if Scope.root_of_nine() != 3 exit(2);
    print("ok\n");
}
