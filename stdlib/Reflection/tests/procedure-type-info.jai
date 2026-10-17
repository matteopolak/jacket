#import "Basic";

Callbacks :: struct #type_info_procedures_are_void_pointers {
    callback: (value: u32) -> u32 #c_call;
}

main :: () {
    for member: type_info(Callbacks).members {
        print("member %, kind %, flags %\n", member.name, member.type.type, member.flags);
        assert(member.type.type == .PROCEDURE || (member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0);
        assert((member.flags & .CONSTANT) == 0);
    }
}
