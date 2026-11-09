// test: compile_fail
#import "Basic"()(ENABLE_ASSERT = false);

run :: () {
    print("unreachable\n");
}
