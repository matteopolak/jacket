#import "Basic";

Default_Instance :: #import "Parameterized_Pack";
Large_Instance :: #import "Parameterized_Pack"(BASE = 12);
Large_Alias :: #import "Parameterized_Pack"(BASE = 12);

main :: () {
    assert(Default_Instance.configured_value() == 5, "module parameter default");
    assert(Large_Instance.configured_value() == 12, "explicit module parameter");
    assert(Default_Instance.next_sequence() == 1, "default instance has independent storage");
    assert(Large_Instance.next_sequence() == 1, "different parameter creates separate instance");
    assert(Large_Alias.next_sequence() == 2, "identical argument list reuses instance");
    print("ok\n");
}
