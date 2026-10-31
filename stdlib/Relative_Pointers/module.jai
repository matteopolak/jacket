// Pointers stored as a small offset from their own address, so a block of memory that holds them can be written to a file or moved and still make sense.
//
// - [Relative_Pointer] holds one pointer; [encode] sets it and [decode] reads it.
// - [Relative_String] holds a string the same way.
// - [write_relative_pointer] and [decode_relative_pointer] work on raw storage.
//
// ```jai
// Node :: struct {
//     next: Relative_Pointer(s16, u8);
//     value: u8;
// }
//
// nodes: [2] Node;
// nodes[1].value = 9;
// encode(*nodes[0].next, *nodes[1].value);
// assert(decode(*nodes[0].next).* == 9);
// ```

Relative_Pointer :: Impl.Relative_Pointer;
Relative_String :: Impl.Relative_String;
decode :: Impl.decode;
encode :: Impl.encode;
decode_relative_pointer :: Impl.decode_relative_pointer;
decode_relative_pointer_static :: Impl.decode_relative_pointer_static;
write_relative_pointer :: Impl.write_relative_pointer;

#scope_file
Impl :: #import "internal/memory/relative-pointers";
