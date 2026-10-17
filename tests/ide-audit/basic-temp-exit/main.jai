using #import "Basic";
Basic :: #import "Basic";
main :: () {
    allocator := Basic.temp; // jailint: allow(unused_variable) the declaration is what the audit inspects
    exit(1);
}
