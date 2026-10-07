// `print("%", p.*)` with a null `p` stops with a null pointer dereference, as `v := p.*;` does,
// on every backend. It used to print `null`: the `Any` points at the place, so nothing read it,
// and print shows a null value pointer as `null`. Taking the address back, `*(p.*)`, reads
// nothing and stays legal.
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
