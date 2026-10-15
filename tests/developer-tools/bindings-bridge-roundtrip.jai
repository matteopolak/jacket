#import "Basic";

native :: #foreign_library "../../target/developer_tools_bridge";

Pair :: struct {
    left: s32;
    right: s32;
}

original_bridge_add :: (left: s32, right: s32) -> s32 #foreign native;
original_bridge_pair :: (result: *Pair, seed: s32) #foreign native;
original_bridge_reference :: () -> *s32 #foreign native;
original_bridge_read_reference :: (value: *s32) -> s32 #foreign native;
original_bridge_write_reference :: (value: *s32, replacement: s32) #foreign native;
original_bridge_c_linkage :: (value: s32) -> s32 #foreign native;

main :: () {
    assert(original_bridge_add(17, 25) == 42);
    pair: Pair;

    original_bridge_pair(*pair, 91);
    assert(pair.left == 91 && pair.right == 92);
    storage := original_bridge_reference();

    assert(storage != null && storage.* == 13);
    assert(original_bridge_reference() == storage);
    assert(original_bridge_read_reference(storage) == 20);
    original_bridge_write_reference(storage, 101);
    assert(storage.* == 101 && original_bridge_read_reference(storage) == 108);
    assert(original_bridge_c_linkage(9) == 27);
    print("ok\n");
}
