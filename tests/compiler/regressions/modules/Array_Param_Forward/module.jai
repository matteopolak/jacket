// Forwards its `[$N] T` parameter to another module that declares the same parameter.
#module_parameters(NAMES: [$N] string = .[]);

#import "Array_Param"(NAMES);

forwarded_count :: () -> int { return name_count() + NAMES.count; }
