// `compiler_set_type_info_flags` changes the descriptor of the program's own struct, whether
// the call is in the run or in a procedure it calls, also when the build is a cross build.
#import "Basic";
#import "Compiler";

helper :: (x: int) -> int {
    return x;
}

Direct :: struct {
    a := 1;
    p := helper;
}

Indirect :: struct {
    a := 1;
    p := helper;
}

reduce :: (T: Type) {
    compiler_set_type_info_flags(T, .PROCEDURES_ARE_VOID_POINTERS);
}

#run compiler_set_type_info_flags(Direct, .PROCEDURES_ARE_VOID_POINTERS);
#run reduce(Indirect);

member_tag :: ($T: Type) -> Type_Info_Tag {
    for type_info(T).members if it.name == "p" return it.type.type;
    return .VOID;
}

main :: () {
    assert(member_tag(Direct) == .POINTER);
    assert(member_tag(Indirect) == .POINTER);
    print("ok\n");
}
