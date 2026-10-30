// A tagged union whose cases are types, like a Rust enum. A jaic extension, not official
// Jai: other Jai compilers have no such module, and it may change between jaic releases. See
// docs/extensions/variant.md.
//
// ```jai
// #import "Extensions/Variant";
//
// Number_Or_Text :: Variant(s64, string);
//
// v := variant(Number_Or_Text, 42);       // the case is the type of the value
// w: Number_Or_Text;                      // zeroed first case
// set(*w, "hi");                          // switches case
//
// if is(v, s64) print("%\n", unwrap(v, s64));
//
// variant_switch(v,                       // every case needs a handler, checked at compile time
//     (it: *s64)    { print("number %\n", it.*); },
//     (it: *string) { print("text %\n", it.*); },
// );
// ```
//
// A Variant is an `index` and the bytes of the largest case, aligned for the strictest one. Two
// cases of one type are not allowed: wrap each in its own struct.

#scope_export

Variant :: struct($types: ..Type) {
    // The position of the live case in `types`.
    index: s32;

    // Declares payload_size, payload_align, case_count, case_infos, case_type_0 ... and the
    // `payload` bytes themselves. Rejects an empty or repeated list of cases.
    #insert -> string {
        return declare_variant(..types);
    };
}

// Makes a Variant holding `value`; its type picks the case. `variant(Shape, Circle.{ 1 })`.
variant :: ($V: Type, value: $T) -> V {
    CASE :: #run case_of(V, T, "variant");

    result: V;

    #if CASE >= 0 set(*result, value);

    return result;
}

// Stores `value` as the live case, replacing whatever was there without releasing it (see
// free_variant). It is a compile-time error when `T` is not a case.
set :: (v: *$V/Variant, value: $T) {
    CASE :: #run case_of(V, T, "set");

    #if CASE >= 0 {
        #assert size_of(T) <= V.payload_size "Variant: a case does not fit its payload";
        #assert align_of(T) <= V.payload_align "Variant: a case is aligned more strictly than its payload";

        memset(v.payload.data, 0, V.payload_size);
        v.index = CASE;

        #if size_of(T) > 0 {
            slot := cast(*T) v.payload.data;
            slot.* = value;
        }
    }
}

// Whether the live case is `T`.
is :: (v: $V/Variant, $T: Type) -> bool {
    CASE :: #run case_of(V, T, "is");

    return v.index == CASE;
}

// The position of the live case in the type's case list.
variant_index :: (v: $V/Variant) -> s32 {
    return v.index;
}

// The type of the live case.
variant_type :: (v: $V/Variant) -> Type {
    return V.types[v.index];
}

// A copy of the `T` payload and true; a zeroed `T` and false when another case is live.
get :: (v: $V/Variant, $T: Type) -> T, bool {
    CASE :: #run case_of(V, T, "get");

    result: T;

    if v.index != CASE return result, false;

    #if size_of(T) > 0 {
        slot := cast(*T) v.payload.data;
        result = slot.*;
    }

    return result, true;
}

// A pointer to the `T` payload inside the variant, or null when another case is live. It stays
// valid until the variant is set again, freed or moved.
get_pointer :: (v: *$V/Variant, $T: Type) -> *T {
    CASE :: #run case_of(V, T, "get_pointer");

    if v.index != CASE return null;

    return cast(*T) v.payload.data;
}

// The `T` payload; asserts that `T` is the live case.
unwrap :: (v: $V/Variant, $T: Type) -> T {
    CASE :: #run case_of(V, T, "unwrap");

    if v.index != CASE {
        assert(false, tprint("unwrap: the Variant holds `%`, not `%`", V.types[v.index], T));
    }

    result: T;

    #if size_of(T) > 0 {
        slot := cast(*T) v.payload.data;
        result = slot.*;
    }

    return result;
}

// The live case is equal: the same index and equal payloads. Strings compare by content, arrays
// element by element, structs member by member; pointers by address.
operator == :: (a: $V/Variant, b: V) -> bool {
    if a.index != b.index return false;

    return values_equal(V.case_infos[a.index], a.payload.data, b.payload.data);
}

operator != :: (a: $V/Variant, b: V) -> bool {
    return !(a == b);
}

// Releases the memory the live case owns, with `free`: strings, resizable arrays (and what their
// elements own), and the same inside structs and nested variants. Views (`[] T`) and pointers are
// not owned and are left alone. Then the variant is zeroed: its first case, zeroed. Only use it
// on payloads that came from the allocator, not string literals.
free_variant :: (v: *$V/Variant) {
    free_variant_value(type_info(V), v);
} @FreesArgument

// ---- printing ----
//
// Basic prints a struct member by member, with no hook for a type of its own. Its one hook is
// `context.print_style.struct_printer`, which Math uses for vectors; this is Variant's.

// A Print_Style struct_printer that writes a Variant as its live payload, and passes everything
// else to the printer that was installed before use_variant_printing (or declines).
variant_struct_printer :: (builder: *String_Builder, any: Any, data: *void) -> bool {
    if any.type && is_variant_type(any.type) {
        append_payload(builder, any);

        return true;
    }

    if previous_printer return previous_printer(builder, any, previous_data);

    return false;
}

// Makes `print("%", variant)` write the payload, from now on in this context. Calling it again
// does nothing; a struct_printer installed earlier still gets the values that are not variants.
use_variant_printing :: () {
    style := *context.print_style;

    if style.struct_printer == variant_struct_printer return;

    previous_printer = style.struct_printer;
    previous_data = style.struct_printer_data;
    style.struct_printer = variant_struct_printer;
    style.struct_printer_data = null;
}

// The payload as text, allocated with context.allocator.
variant_to_string :: (v: $V/Variant) -> string {
    style := *context.print_style;
    saved_printer := style.struct_printer;
    saved_data := style.struct_printer_data;

    use_variant_printing();

    defer {
        style.struct_printer = saved_printer;
        style.struct_printer_data = saved_data;
    }

    return sprint("%", v);
}

// ---- reflection, for modules that code over variants (Json) ----

// Whether `info` is an instantiation of Variant.
is_variant_type :: (info: *Type_Info) -> bool {
    if info.type != .STRUCT return false;

    found_infos := false;
    found_payload := false;

    for (cast(*Type_Info_Struct) info).members {
        if it.name == "case_infos" && (it.flags & .CONSTANT) != 0 found_infos = true;
        if it.name == "payload" found_payload = true;
    }

    return found_infos && found_payload;
}

// The types of the cases of a Variant, in order.
variant_case_infos :: (info: *Type_Info) -> [] *Type_Info {
    for (cast(*Type_Info_Struct) info).members {
        if it.name != "case_infos" || it.offset_into_constant_storage < 0 continue;

        storage := (cast(*Type_Info_Struct) info).constant_storage;

        return (cast(*[] *Type_Info) (storage.data + it.offset_into_constant_storage)).*;
    }

    return .[];
}

// ---- implementation ----

#load "layout.jai";
#load "reflect.jai";
#load "switch.jai";

#scope_module

previous_printer: (builder: *String_Builder, any: Any, data: *void) -> bool;
previous_data: *void;

append_payload :: (builder: *String_Builder, any: Any) {
    info := cast(*Type_Info_Struct) any.type;
    base := cast(*u8) any.value_pointer;
    cases := variant_case_infos(info);
    payload: s64;

    for info.members if it.name == "payload" payload = it.offset_in_bytes;

    index := (cast(*s32) base).*;

    if index < 0 || index >= cases.count {
        append(builder, "(invalid variant)");

        return;
    }

    item: Any;
    item.type = cases[index];
    item.value_pointer = base + payload;

    print(builder, "%", item);
}

#import "Basic";
#import "Compiler";
