// A struct literal field overriding a pointer-holding default (a string) replaces the default's
// relocation in the constant image; native code used to keep the default's data pointer.
#import "Basic";

Info :: struct {
    a: int = -10;
    flavor := "vanilla";
    n: u32 = 1;
    cone := "waffle";
}

main :: () {
    x := Info.{ flavor = "chocolate", n = 3 };
    y: Info;
    if x.flavor != "chocolate" || y.flavor != "vanilla" || x.cone != "waffle" {
        print("FAIL % %\n", x, y);
        exit(1);
    }
    print("ok\n");
}
