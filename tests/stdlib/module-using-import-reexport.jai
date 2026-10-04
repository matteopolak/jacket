#import "Basic";
#import "Reexport_Outer";

main :: () {
    t := Inner_Thing.{number = 42};
    assert(t.number + inner_value() == 49);
    print("ok\n");
}
