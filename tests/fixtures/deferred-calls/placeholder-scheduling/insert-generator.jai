#import "Basic";
C :: #import "Compiler";

#placeholder Source;
binding_scope :: #code Source;
prefixes: int;

generate :: () -> string {
    prefixes += 1;
    return Source;
}

#insert #run generate();
#run C.add_build_string("Source :: \"answer :: () -> int {return 42;}\";", -1, binding_scope);
#assert prefixes == 1;
#assert answer() == 42;

main :: () {
    print("generator once\n");
}
