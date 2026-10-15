// Exports the members of a global procedure table with `using`, the way GL does.
Procs :: struct {
    answer: () -> int #c_call;
}

procs: Procs;
using procs;

load_procs :: () {
    procs.answer = xx forty_two;
}

#scope_file
forty_two :: () -> int #c_call {
    return 42;
}
