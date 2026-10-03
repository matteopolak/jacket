provider :: #foreign_library "./provider.a";
State :: struct { total:s64; status:s32; }
counter:s64 #elsewhere provider "jai_test_external_counter";
state:State #elsewhere provider "jai_test_external_state";
counter_address :: ()->*s64 #foreign "jai_test_external_counter_address";
state_address :: ()->*State #foreign "jai_test_external_state_address";
provider_read :: ()->s32 #foreign "jai_test_external_provider_read";

main :: ()->int {
    if counter!=20 || state.total!=1 || state.status!=3 return 1;
    if *counter!=counter_address() || *state!=state_address() return 2;
    counter+=1;
    state.total=21;
    state.status=7;
    return provider_read();
}
