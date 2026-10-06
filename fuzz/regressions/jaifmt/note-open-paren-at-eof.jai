hash_table_find_returns_value_then_found_flag :: () {
    expect_program_output("test/modules/hash_table/find_returns_value_then_found_value.jai", "true 520-1234 false \n")#caller_location;
} @Testassert(Procedure
