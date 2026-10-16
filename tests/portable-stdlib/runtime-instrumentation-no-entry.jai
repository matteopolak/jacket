// Standalone object configuration: hooks exist, but no system entry invokes them.
R :: #import "Runtime_Support";

#program_export "owned_no_entry"
call_default_hooks :: () {
    R.__instrumentation_first();
    R.__instrumentation_second();
}
