#import "Basic";
#import "Xar";
X :: #import "Xar";
Again :: #import "Xar";

main :: () {
    values: Xar(s64, 1, 4);

    defer xar_reset(*values);
    pointer := xar_add(*values, cast(s64) 17);

    xar_add(*values, cast(s64) 23);
    assert(values[0] == 17 && values[1] == 23);
    address := *values[0];

    assert(address == pointer);
    assert(type_info(X.Xar(s64, 1, 4)) == type_info(Again.Xar(s64, 1, 4)));
    values[1] = 31;
    assert(values[1] == 31);
    print("ok\n");
}
