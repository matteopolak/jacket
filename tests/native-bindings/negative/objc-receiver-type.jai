O :: #import "Objective_C";

invalid :: (self: *u8, selector: O.Selector) #c_call {}

main :: () {
    O.type_check_objc_method_type(type_info(type_of(invalid)));
}
