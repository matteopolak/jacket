// Original metadata contract used by reviewed Basic default elision.
#import "Basic";

Box :: struct(T: Type = u8, amount: int = size_of(T)) {
    item: T;
}

main :: () {
    first := type_info(Box(u32, 4));
    second := type_info(Box(u8, 9));
    source := first.polymorph_source_struct;

    assert(source != null && source == second.polymorph_source_struct);
    assert(source.parameters.count == 2);
    assert(first.parameters.count == 2 && first.specified_parameters.count == 2);

    for parameter: source.parameters {
        assert(parameter.type != null && parameter.offset_into_constant_storage >= 0);
    }

    declared := source.parameters[1];
    resolved := first.parameters[1];
    original := (cast(*int) (source.constant_storage.data + declared.offset_into_constant_storage)).*;
    evaluated := (cast(*int) (first.constant_storage.data + resolved.offset_into_constant_storage)).*;

    assert(original == 1 && evaluated == 4);
    print("ok\n");
}
