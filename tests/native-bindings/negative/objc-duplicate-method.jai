O :: #import "Objective_C";

DuplicateMethodObject :: struct {
    using parent: O.NSObject;
}

method :: (self: O.id, selector: O.Selector) #c_call {}

main :: () {
    created := O.objc_create_class(DuplicateMethodObject, O.NSObject);
    O.objc_add_instance_method(created, method, "duplicate");
    O.objc_add_instance_method(created, method, "duplicate");
}
