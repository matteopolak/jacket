O :: #import "Objective_C";

invalid :: (self: O.id, selector: *s32) #c_call {}

main :: () {
    O.type_check_objc_method_type(type_info(type_of(invalid)));
}
