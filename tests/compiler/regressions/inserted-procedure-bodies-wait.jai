// A procedure that a file-scope `#insert` declares has its body checked when something calls it.
// A generated overload nothing calls may name things that do not exist.
#import "Basic";

Slot :: struct {
    value: int;
}

store :: (slot: *Slot, value: int) {
    slot.value = value;
}

generate :: () -> string {
    // The overload for `float` names a buffer that was never generated.
    return "update :: (slot: *Slot, value: float) { Missing_Buffer[0] = value; }\nupdate :: (slot: *Slot, value: int) { store(slot, value); }\n";
}

#insert #run generate();

main :: () {
    slot: Slot;
    update(*slot, 7);
    assert(slot.value == 7);
}
