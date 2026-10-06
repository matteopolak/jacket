// rules: prelude.1 prelude.2 prelude.3 prelude.4 prelude.5
// Pins the claims of docs/metaprogramming/prelude-and-runtime-support.md. No module is
// imported before the prelude.1 checks: Preload and Runtime_Support are there implicitly.

// prelude.1: Preload declarations need no import.
Pair :: struct {
    left: int;
    right: int;
}
PAIR_INFO :: #run -> int {
    info := cast(*Type_Info_Struct) type_info(Pair);
    return info.members.count;
};
#assert PAIR_INFO == 2;

// prelude.2: modules see them too (Rules_Prelude imports nothing).
Prelude_User :: #import "Rules_Prelude";

// prelude.3: a program may import Runtime_Support itself with other parameters and gets a
// separate instance whose procedures work.
Own_Runtime :: #import "Runtime_Support"(DEFINE_SYSTEM_ENTRY_POINT = false, DEFINE_INITIALIZATION = false, ENABLE_BACKTRACE_ON_CRASH = false);

// prelude.4: the Context starts with Context_Base (as `base`), then #add_context members in
// load order.
#add_context rules_first_extra: int;
#add_context rules_second_extra: float;

#import "Basic";

member_index :: (info: *Type_Info_Struct, name: string) -> int {
    for info.members if it.name == name return it_index;
    return -1;
}

main :: () {
    // prelude.1: the entry point and context come from the implicit Runtime_Support.
    assert(PAIR_INFO == 2, "prelude.1: Type_Info from Preload");
    alloc_ok: Allocator = context.allocator;
    assert(alloc_ok.proc != null, "prelude.1: context is initialized by Runtime_Support");

    // prelude.2
    assert(Prelude_User.struct_member_count(Pair) == 2, "prelude.2: Type_Info in a module");
    assert(Prelude_User.allocator_is_set(), "prelude.2: Allocator and context in a module");
    src := u8.[1, 2, 3];
    dst: [3] u8;
    Prelude_User.copy_bytes(dst.data, src.data, 3);
    assert(dst[2] == 3, "prelude.2: memcpy in a module");

    // prelude.3
    assert(Own_Runtime.c_style_strlen("four\0tail".data) == 4, "prelude.3: own Runtime_Support instance");

    // prelude.4
    info := type_info(Context);
    assert(info.members.count > 0 && info.members[0].name == "base", "prelude.4: Context_Base first");
    assert(info.members[0].type == type_info(Context_Base), "prelude.4: base is a Context_Base");
    first := member_index(info, "rules_first_extra");
    second := member_index(info, "rules_second_extra");
    assert(first > 0 && second > first, "prelude.4: #add_context members in load order");
    context.rules_first_extra = 5;
    assert(context.rules_first_extra == 5, "prelude.4: added member is usable");

    // prelude.5: the intrinsics need no import.
    buffer: [8] u8;
    memset(buffer.data, 7, 8);
    assert(buffer[0] == 7 && buffer[7] == 7, "prelude.5: memset");
    other: [8] u8;
    memcpy(other.data, buffer.data, 8);
    assert(memcmp(other.data, buffer.data, 8) == 0, "prelude.5: memcpy and memcmp");
    other[3] = 9;
    assert(memcmp(other.data, buffer.data, 8) != 0, "prelude.5: memcmp sees a difference");
    slot := 10;
    ok, old := compare_and_swap(*slot, 10, 11);
    assert(ok && old == 10 && slot == 11, "prelude.5: compare_and_swap succeeds");
    ok, old = compare_and_swap(*slot, 10, 12);
    assert(!ok && old == 11 && slot == 11, "prelude.5: compare_and_swap fails on mismatch");
}
