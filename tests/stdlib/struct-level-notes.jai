// Notes written on a struct (`S :: struct @thing { ... }`) show up in `type_info(S).notes`, next to the
// per-member notes that were already supported.
// rules: decl.20 decl.21 note.7 note.8
#import "Basic";

Tagged :: struct @thing @"two words" {
    a: int @member_note;
    b: float;
}

Plain :: struct {
    x: int;
}

Generic :: struct(T: Type) @generic {
    value: T;
}

Variant :: union @shared {
    i: int;
    f: float;
}

main :: () {
    info := type_info(Tagged);
    if info.notes.count != 2 exit(1);
    if info.notes[0] != "thing" exit(2);
    if info.notes[1] != "two words" exit(3);
    if info.members[0].notes.count != 1 || info.members[0].notes[0] != "member_note" exit(4);
    if info.members[1].notes.count != 0 exit(5);
    if type_info(Plain).notes.count != 0 exit(6);
    if type_info(Generic(int)).notes.count != 1 || type_info(Generic(int)).notes[0] != "generic" exit(7);
    if type_info(Variant).notes.count != 1 || type_info(Variant).notes[0] != "shared" exit(8);
    #run {
        notes := type_info(Tagged).notes;
        assert(notes.count == 2 && notes[0] == "thing");
    };
    print("ok\n");
}
