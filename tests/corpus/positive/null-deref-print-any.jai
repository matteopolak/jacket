// `print("%", p.*)` with a null `p` stops with a null pointer dereference, as `v := p.*;` does,
// on every backend: boxing `p.*` into an `Any` reads nothing, so `print` is what finds the null
// value pointer, and it stops instead of showing `null`. Taking the address back, `*(p.*)`,
// reads nothing and stays legal.
#import "Basic";

main :: () {
    k := 7;
    p := *k;
    print("%\n", p.*);
    p = null;
    back := *(p.*);
    print("%\n", back == null);
    print("%\n", p.*);
    print("unreachable\n");
}
