// test: skip_on(.WASM, mode = .INTERP, reason = "the wasm sandbox has no PATH variable")
// PATH is set on every desktop platform the tests run on; the other name is not set anywhere.
#import "Basic";
#import "Extensions/Environment";

UNSET :: "JACKET_ENVIRONMENT_TEST_UNSET_VARIABLE";

main :: () {
    assert(has_environment_variable("PATH") && environment_variable("PATH").count > 0, "PATH is not set");
    assert(!has_environment_variable(UNSET) && environment_variable(UNSET) == "", "an unset variable has a value");

    print("ok\n");
}
