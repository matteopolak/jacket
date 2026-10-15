#import "Basic";
#import "Sound_Player";

main :: () {
    a, b: Audio_Sample_Base;
    a.channels[0] = 0.25;
    a.channels[7] = 0.5;
    b.channels[0] = 0.75;
    sum := add(*a, *b, 1);

    assert(sum.channels[0] == 1);
    assert(sum.channels[7] == 0);
    scaled := multiply(*a, 2.0);

    assert(scaled.channels[0] == 0.5 && scaled.channels[7] == 1);
    plus_equals(*a, *b, 1);
    times_equals(*a, 0.5);
    assert(a.channels[0] == 0.5 && a.channels[7] == 0.25);
    index, remainder := get_icursor_and_remainder(3.25);

    assert(index == 3 && remainder == 0.25);
    index, remainder = get_icursor_and_remainder(-0.25);
    assert(index == -1 && remainder == 0.75);
    print("ok\n");
}
