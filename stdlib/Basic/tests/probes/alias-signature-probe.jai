#import "Basic";
Facade :: #import "alias-signature-facade";

main :: () {
    assert(Facade.measure() == 7);
    assert(Facade.measure(count = 19) == 19);
    assert(Facade.measure(text = "abcd") == 4);
    print("ok\n");
}
