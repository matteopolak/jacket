// The layout of a `string`: a byte count and a pointer to the bytes.
Newstring :: struct {
    count: s64;
    data: *u8;
}

// The layout of any `[] T` with the element type erased, for code that handles array views of
// every type through one pointer cast.
//
// ```jai
// numbers := int.[3, 1, 4];
// view: [] int = numbers;
// raw := cast(*Array_View_64) *view;
// assert(raw.count == 3);
// assert(raw.data == cast(*u8) numbers.data);
// ```
Array_View_64 :: struct {
    count: s64;
    data: *u8;
}

// The layout of any `[..] T` with the element type erased. `allocated` is the capacity in elements
// and `allocator` is where `data` came from. Basic's `array_reserve` grows one through this view.
Resizable_Array :: struct {
    count: s64;
    data: *void;
    allocated: s64;

    allocator: Allocator;
}

// The layout of an `Any`: the value's type and a pointer to it.
Any_Struct :: struct {
    type: *Type_Info;
    value_pointer: *void;
}

// A procedure pointer and the state to call it with, both untyped.
Procedure_With_Data :: struct {
    proc: *void;
    data: *void;
}

// How a `for` loop calls a `for_expansion` macro: `for *` sets `.POINTER` and `for <` sets
// `.REVERSE`. The macro reads them from its `flags` parameter.
For_Flags :: enum_flags u8 {
    POINTER :: 0x1;
    REVERSE :: 0x2;
}
