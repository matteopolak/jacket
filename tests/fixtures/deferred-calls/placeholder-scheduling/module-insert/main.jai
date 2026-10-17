C :: #import "Compiler";
M :: #import "Deferred_Module";

#run {
    C.add_build_string("Body :: \"answer :: () -> int {return 42;}\";", -1, M.binding_scope);
}

#assert M.answer() == 42;

main :: () {
    value := M.answer(); // jailint: allow(unused_variable) the scheduled call is what is tested
}
