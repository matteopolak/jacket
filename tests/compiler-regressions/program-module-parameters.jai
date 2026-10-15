// Program parameters (the second #module_parameters list) configure the one shared
// instance: a plain import and a configuring import see the same types and values.
// rules: modparam.4
// jailint: allow-file(unused_variable) -- the declarations only type-check the API.
#import "Basic";
#import "Program_Param";
Configured :: #import "Program_Param"()(VERBOSE = true);

main :: () {
    m: Message;
    n: Configured.Message = m;
    assert(is_verbose() && Configured.is_verbose());
}
