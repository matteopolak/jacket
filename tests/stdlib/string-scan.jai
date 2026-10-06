// String.scan / scan2 / atof.
#import "Basic";
#import "String";

Vec :: struct {
    x, y, z: float;
}

main :: () -> s64 {
    ok, values := scan("(%f, %i, %b, %s)", "(1.5, 42, true, \"hi\")");
    if !ok || values.count != 4 return 1;
    if (cast(*float) values[0].value_pointer).* != 1.5 return 2;
    if (cast(*int) values[1].value_pointer).* != 42 return 3;
    if !(cast(*bool) values[2].value_pointer).* return 4;
    if (cast(*string) values[3].value_pointer).* != "hi" return 5;
    bad, _ := scan("a%i", "b7");
    if bad return 6;

    v: Vec;
    if !scan2("(1 2.5 -3)", "(% % %)", *v.x, *v.y, *v.z) return 7;
    if v.x != 1 || v.y != 2.5 || v.z != -3 return 8;
    if scan2("(1 2)", "(% % %)", *v.x, *v.y, *v.z) return 9;
    if atof("3.25") != 3.25 return 10;
    print("ok\n");
    return 0;
}
