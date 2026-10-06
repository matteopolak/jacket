// rules: ext.2
// Long_Double passes to and returns from C functions declared with it (here libm's sqrtl),
// whatever the host's C `long double` is.
#import "Basic";
#import "Jaic_Extensions";

libm :: #system_library "libm";
sqrtl :: (x: Long_Double) -> Long_Double #foreign libm;
fmaxl :: (x: Long_Double, y: Long_Double) -> Long_Double #foreign libm;

main :: () {
    one: Long_Double = 1;
    quarter := one / 4;
    root := sqrtl(quarter);
    assert(root == one / 2, "sqrtl(0.25)");
    assert(type_of(root) == Long_Double, "result type");
    assert(fmaxl(quarter, root) == root, "two Long_Double arguments");
    assert(tprint("%", sqrtl(one * 2)) == "1.414214", "printed through float64");
    print("ok\n");
}
