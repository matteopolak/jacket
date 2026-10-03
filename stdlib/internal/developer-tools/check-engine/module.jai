// Private engine; the public facade controls conditional compatibility exports.
#module_parameters(CHECK_BINDINGS := true, DEBUG_BINDINGS := false);

C_Porting_Check_Settings :: struct {
    skip_modules := false;
    check_treatment_of_arrays_as_pointers_in_procedure_calls := true;
    check_treatment_of_arrays_as_pointers_in_assignments := true;
    check_intermediate_cast_to_int_when_adding_small_integers := true;
}

#load "../porting-check.jai";
#load "../check-bindings.jai";
