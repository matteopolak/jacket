State :: struct { total:s64; status:s32; }
jai_test_external_counter:s64 #elsewhere;
jai_test_external_state:State #elsewhere;
counter_address :: ()->*s64 #foreign "jai_test_external_counter_address";
state_address :: ()->*State #foreign "jai_test_external_state_address";
provider_read :: ()->s32 #foreign "jai_test_external_provider_read";

main :: ()->int {
    if jai_test_external_counter!=20 || jai_test_external_state.total!=1 || jai_test_external_state.status!=3 return 1;
    if *jai_test_external_counter!=counter_address() || *jai_test_external_state!=state_address() return 2;
    jai_test_external_counter+=1;
    jai_test_external_state.total=21;
    jai_test_external_state.status=7;
    return provider_read();
}
