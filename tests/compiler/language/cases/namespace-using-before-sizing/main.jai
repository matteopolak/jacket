#import "Basic";

Registry :: struct {
    items: [2] *Entry;
}

registry: Registry;
using registry;

Space :: #import "Entry_Space";

using Space;

main :: () {
    entry := Entry.{ id = 5 };
    registry.items[0] = *entry;
    assert(registry.items[0].id == 5, "a global sized before `using Space;` names what that brings in");
    print("ok\n");
}
