// A for_expansion may forward its body to another for_expansion: `it` / `it_index` come from the
// macro that inserts the body.
// rules: macro.11
#import "Basic";

A :: struct {
    x: [3] int;
}

for_expansion :: (a: *A, body: Code, flags: For_Flags) #expand {
    for i: 0..a.x.count - 1 {
        `it := a.x[i];
        `it_index := i;
        #insert body;
    }
}

B :: struct {
    a: A;
}

for_expansion :: (b: *B, body: Code, flags: For_Flags) #expand {
    for_expansion(*b.a, body, flags);
}

main :: () {
    b: B;
    b.a.x = .[1, 2, 3];
    s := 0;
    for b s += it;
    if s != 6 exit(1);
    t := 0;
    for v, i: b t += v * i;
    if t != 8 exit(2);
    print("ok\n");
}
