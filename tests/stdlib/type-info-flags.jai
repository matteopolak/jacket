// compiler_set_type_info_flags and the #type_info_* struct directives reduce a struct's Type_Info.
#import "Basic";
#import "Compiler";

helper :: (x: int) -> int { return x; }

Plain :: struct { a := 1; p := helper; }
Voided :: struct { a := 1; p := helper; }
Hidden :: struct { a := 1; p := helper; }
Directive :: struct #type_info_procedures_are_void_pointers { a := 1; p := helper; }
Empty :: struct #type_info_none { a := 1; }

#run compiler_set_type_info_flags(Voided, .PROCEDURES_ARE_VOID_POINTERS);
#run compiler_set_type_info_flags(Hidden, .NO_TYPE_INFO);

proc_tag :: ($T: Type) -> Type_Info_Tag {
    for type_info(T).members if it.name == "p" return it.type.type;
    return .VOID;
}

check :: (cond: bool, what: string) { if !cond { print("FAIL: %\n", what); exit(1); } }

main :: () {
    check(proc_tag(Plain) == .PROCEDURE, "plain keeps the procedure type");
    check(proc_tag(Voided) == .POINTER, "flag: procedures are void pointers");
    check(proc_tag(Directive) == .POINTER, "directive: procedures are void pointers");
    check(type_info(Hidden).members.count == 0, "flag: no type info");
    check(type_info(Empty).members.count == 0, "directive: no type info");
    check(type_info(Plain).members.count == 2, "plain members");
    print("ok\n");
}
