O :: #import "Objective_C";

Malformed :: struct {
    isa: u64;
}

main :: () {
    O.has_isa_member(type_info(Malformed));
}
