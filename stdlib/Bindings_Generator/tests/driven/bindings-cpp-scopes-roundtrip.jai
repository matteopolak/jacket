#import "Basic";
G :: #import "developer-tools-cpp-scopes-generated";

double_value :: (value: s64) -> s64 #c_call {
    return value * 2;
}

main :: () {
    original := G.AlphaSample.{ value = 11 };
    result := G.alpha_adjust(original, 7);

    assert(result.value == 18 && original.value == 11);
    real := G.BetaSample.{ value = 2.5 };

    assert(G.beta_adjust(real, 4) == 10);
    assert(G.alpha_repeat(9) == 20);
    assert(G.alpha_scale(9) == 27);
    bytes: [3] u8 = .[65, 66, 0];
    pointer := bytes.data;

    assert(G.alpha_identity(pointer) == pointer);
    large: s64 = 0x100000013;

    assert(G.alpha_long_identity(*large) == *large);
    assert(G.alpha_invoke(double_value, 21) == 47);
    assert(G.alpha_c_identity(9) == 36);
    print("ok\n");
}
