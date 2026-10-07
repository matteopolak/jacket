// Jai #c_call procedures taking and returning C `long double` (alone and in a struct), called
// from C (interpreter and native).
#import "Basic";

longdouble :: #library "liblongdouble";

#load "types.jai";

difference :: (a: Long_Double, b: Long_Double) -> Long_Double #c_call {
    return a - b;
}

triple_box :: (b: LdBox, k: s32) -> LdBox #c_call {
    return .{ b.v * k };
}

main :: () {
    third := ld_third();
    print("apply % %\n", ld_apply(difference, 10, 2.5), ld_apply(difference, third, -third) == third + third);
    print("box_apply %\n", box_apply(triple_box, .{ 1.5 }).v);
}
