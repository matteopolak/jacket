// A struct's type info lists every procedure constant, used or not, with the CONSTANT flag and an
// offset_in_bytes of -1 (a constant takes no space in an instance).
// rules: decl.23
#import "Basic";

Hooks :: struct {
    value: int;
    LIMIT :: 5;
    unused_hook :: (x: int) -> int {
        return x + 1;
    }
    generic :: (x: $T) {}
    expander :: (n: int) #expand {}
    used_hook :: () {}
}

find :: (name: string) -> *Type_Info_Struct_Member {
    for *type_info(Hooks).members if it.name == name return it;
    return null;
}

main :: () {
    Hooks.used_hook();
    for name: string.["unused_hook", "generic", "expander", "used_hook", "LIMIT"] {
        member := find(name);
        assert(member != null, "member is listed");
        assert((member.flags & .CONSTANT) != 0, "member is a constant");
        assert(member.offset_in_bytes == -1, "constant has no offset");
    }
    assert(find("value").offset_in_bytes == 0 && (find("value").flags & .CONSTANT) == 0);
    assert(find("unused_hook").type.type == .PROCEDURE);
    assert(find("generic").type.type == .PROCEDURE);
    print("ok\n");
}
