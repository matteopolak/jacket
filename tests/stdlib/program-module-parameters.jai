// Program parameters (the second #module_parameters list) configure the one shared
// instance: a plain import and a configuring import see the same types and values.
#import "Basic";
#import "Program_Param";
Configured :: #import "Program_Param"()(VERBOSE = true);

main :: () {
    m: Message;
    n: Configured.Message = m;
    assert(is_verbose() && Configured.is_verbose());
}
