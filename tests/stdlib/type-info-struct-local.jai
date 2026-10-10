// A struct declared in a procedure or in another struct is LOCAL in its type info; a top-level one is not.
#import "Basic";

Outer :: struct {
    Inside :: struct {
        a: int;
    }
}

Top :: struct {
    a: int;
}

main :: () {
    Scoped :: struct {
        a: int;
    }
    flags :: (t: Type) -> Struct_Status_Flags {
        return (cast(*Type_Info_Struct) t).status_flags;
    }
    assert((flags(Scoped) & .LOCAL) != 0);
    assert((flags(Outer.Inside) & .LOCAL) != 0);
    assert((flags(Top) & .LOCAL) == 0);
    assert((flags(Outer) & .LOCAL) == 0);
    print("ok\n");
}
