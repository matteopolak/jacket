// Independent ABI safety: a Jai implicit context is not an Objective-C argument.
#import "Basic"; // jailint: allow(unused_import) the import is part of the rejected program
O :: #import "Objective_C";

invalid :: (self: O.id, selector: O.Selector) {}

main :: () {
    O.type_check_objc_method_type(type_info(type_of(invalid)));
}
