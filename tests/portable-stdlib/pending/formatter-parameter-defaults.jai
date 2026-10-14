// Compiler handoff: generic template descriptors must expose parameter defaults.
#import "Basic";

Box :: struct(T: Type = int, amount: int = 3) {
    value: T;
}

main :: () {
    instance := cast(*Type_Info_Struct) type_info(Box(int, 3));

    assert(instance.specified_parameters.count == 2);
    source := instance.polymorph_source_struct;

    assert(source != null);
    assert(source.parameters.count == 2);

    // Defaults must carry usable types and constant-storage offsets, allowing
    // Basic to compare each instantiated argument with its declared default.
    for parameter: source.parameters {
        assert(parameter.type != null && parameter.offset_into_constant_storage >= 0);
    }
}
