#import "Basic";
B :: #import "Bindings_Generator";

S :: type_of(context._bindings_generator_session.*);

main :: () {
    generator: B.Generator;
    generator.pointer_size = size_of(*void);
    session := S.open(*generator, B.TARGET_OS);

    defer S.close(*session);

    push_context {
        S.activate(*generator, null, *session);
        integer: B.CType;
        integer.size = 4;
        integer.number_flags = ._32BIT | .SIGNED;
        fixed: B.CType;
        fixed.array_element_type = *integer;
        fixed.array_element_count = 3;
        fixed.size = 12;
        fixed.flags = .SHARED;
        converted, changed := B.maybe_convert_fixed_array_type(*fixed);

        assert(changed && converted != *fixed && converted.pointer_to == *fixed);
        assert((converted.flags & .INSERTED) != 0 && converted.size == generator.pointer_size);
        assert(fixed.array_element_count == 3 && fixed.array_element_type == *integer && fixed.flags == .SHARED);
        alias: B.Typedef;
        alias.type = *fixed;
        aliased: B.CType;
        aliased.type_of_typedef = *alias;
        from_alias, alias_changed := B.maybe_convert_fixed_array_type(*aliased);

        assert(alias_changed && from_alias.pointer_to == *aliased);
        incomplete: B.CType;
        incomplete.array_element_type = *fixed;
        nested, nested_changed := B.maybe_convert_fixed_array_type(*incomplete);

        assert(nested_changed && nested.array_element_type.pointer_to == *fixed);
        assert(nested.array_element_count == -1 && incomplete.array_element_type == *fixed);
        unchanged, not_changed := B.maybe_convert_fixed_array_type(*integer);

        assert(!not_changed && unchanged == *integer);
        alias.name = "va_list";
        exempt, exempt_changed := B.maybe_convert_fixed_array_type(*aliased);

        assert(!exempt_changed && exempt == *aliased);
    }

    print("ok\n");
}
