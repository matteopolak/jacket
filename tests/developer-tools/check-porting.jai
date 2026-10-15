#import "Basic";

#if OS == .WINDOWS {
    C :: #import "Check";
    #import "Compiler";

    #run {
        settings: C.C_Porting_Check_Settings;

        assert(!settings.skip_modules);
        assert(settings.check_treatment_of_arrays_as_pointers_in_procedure_calls);
        assert(settings.check_treatment_of_arrays_as_pointers_in_assignments);
        assert(settings.check_intermediate_cast_to_int_when_adding_small_integers);
        fixed: Code_Node;
        fixed.type = type_info([4] u8);
        assert(C.is_of_fixed_array_type(*fixed));
        fixed.type = type_info([] u8);
        assert(!C.is_of_fixed_array_type(*fixed));
        module: Message_Import;
        module.module_type = .FILE;
        file: Message_File;
        file.enclosing_import = *module;
        fixed.enclosing_load = *file;
        assert(!C.should_skip(*fixed, settings));
        settings.skip_modules = true;
        assert(C.should_skip(*fixed, settings));
        module.module_type = .MAIN_PROGRAM;
        assert(!C.should_skip(*fixed, settings));
    }
}

main :: () {
    print("ok\n");
}
