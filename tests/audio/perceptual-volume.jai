#import "Basic";
#import "Math";
#import "Sound_Player";

main :: () {
    assert(abs(perceptual_to_linear(0.5) - sqrt(0.1)) < 0.00001);
    assert(perceptual_to_linear(0) == 0);
    assert(perceptual_to_linear(1) == 1);
    assert(perceptual_to_linear(2) > 1);
    print("ok\n");
}
