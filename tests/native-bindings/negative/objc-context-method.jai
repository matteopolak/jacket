// Independent ABI safety: a Jai implicit context is not an Objective-C argument.
#import "Basic";
O :: #import "Objective_C";

invalid :: (self: O.id, selector: O.Selector) {}

main :: () {
    O.type_check_objc_method_type(type_info(type_of(invalid)));
}
