// C `long double` arguments, results and struct members through #foreign (interpreter and
// native). The same output is expected whatever the target's long double is.
#import "Basic";

longdouble :: #library "liblongdouble";

#load "types.jai";

main :: () {
    third := ld_third();
    // Only a wide long double keeps bits past float64's, and C must agree with Jai about them.
    print("third % wide %\n", third, ld_beyond_double(third) == cast(s32) LONG_DOUBLE_IS_WIDE);
    print("add % mix %\n", ld_add(third, third) == third + third, ld_mix(2, 0.5, 1.25, 2.5, 3));
    print("to_double %\n", ld_to_double(third) == cast(float64) third);
    box := box_scale(.{ 1.5 }, 4);
    tagged := tagged_bump(.{ 5, third });
    pair := pair_swap(.{ 1, third });
    print("box % tagged % % pair % %\n", box.v, tagged.tag, tagged.v == third * 2, pair.a == third, pair.b);
    print("spill %\n", ld_spill(1, 2, 3, 4, 5, 6, 7, 8, 9, third, 3, 0.25));
}
