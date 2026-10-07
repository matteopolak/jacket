// A scalar module argument takes the parameter's written type (`SAMPLES: s32`).
// rules: modparam.6
#import "Basic";
#import "Typed_Param"(SAMPLES = 0);

main :: () {
    assert(samples() == 0);
}
