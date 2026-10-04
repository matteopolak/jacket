// `#insert` of a string inside an enum body adds members.
#import "Basic";

NAMES :: string.["Alpha", "Beta"];

Kind :: enum u8 {
    None;
    #insert -> string {
        builder: String_Builder;
        for NAMES print_to_builder(*builder, "%;\n", it);
        return builder_to_string(*builder);
    }
    Last;
}

main :: () {
    if Kind.Alpha != 1 exit(1);
    if Kind.Beta != 2 exit(2);
    if Kind.Last != 3 exit(3);
    print("%\n", Kind.Beta);
}
