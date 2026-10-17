C :: #import "Compiler";

#placeholder Body;
binding_scope :: #code Body;
#insert Body;

#run {
    C.add_build_string("Body :: \"answer :: () -> int {return 42;}\";", -1, binding_scope);
}

#assert answer() == 42;

main :: () {
    value := answer(); // jailint: allow(unused_variable) the scheduled call is what is tested
}
