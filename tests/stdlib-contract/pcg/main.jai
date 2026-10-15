// Compatibility vector for the public state-pointer PCG interface.
#import "Basic";
PCG :: #import "PCG";

main :: () {
    state: u64;

    PCG.random_seed(*state, 42);
    assert(state == 0x91778aed87ee5eb1);
    assert(PCG.random_get(*state) == 0x75830bbd);
    assert(state == 0x39b7f8a5c64cf56c);
    assert(PCG.random_get(*state) == 0x0e6dfdb2);
    assert(state == 0x69afc5a5e88b394b);
    assert(PCG.random_get_within_bound(*state, 1) == 0);
    assert(PCG.random_get_within_bound(*state, 17) < 17);
    print("ok\n");
}
