// A tagged union. Each case has a name and a type; two cases may share a type. `which` says
// which case is live, and the zero value is `.NONE`.
//
// ```jai
// #import "Extensions/Variant";
//
// Contact :: Variant(struct { email: string; phone: string; });   // named cases
// Tree    :: Variant(Leaf, *Branch);                               // cases named after their types
//
// c: Contact;                             // c.which == .NONE
// c.which = .PHONE; c.phone = "555";      // the cases are fields
//
// t: Tree;
// set(*t, Leaf.{ 1 });                    // by type, when only one case has it
// if t.which == .LEAF print("%\n", t.leaf);
//
// if #complete c.which == {
//     case .NONE;
//     case .EMAIL; print("mail %\n", c.email);
//     case .PHONE; print("call %\n", c.phone);
// }
// ```
//
// A type list names each case after its type: `HttpRequest` and `Http_Request` become
// `http_request`, `*Branch` becomes `branch`, and a built-in type gets a trailing `_` (`s64_`,
// `string_`). The tag is the name in upper case without that `_`: `.HTTP_REQUEST`, `.S64`.

#scope_export

// A value that holds one of its cases at a time, or none. It is the tag plus the largest case:
// `which` is a `u8` enum (`u16` past 255 cases), and the cases share the bytes after it.
//
// ```jai
// #import "Extensions/Variant";
//
// Id :: Variant(s64, string);
// id: Id;
// assert(id.which == .NONE);
// id.which = .STRING;
// id.string_ = "abc";
// assert(id.string_ == "abc");
// ```
Variant :: struct($types: ..Type) {
    // Declares the `Which` enum (NONE and one tag per case), case_count, case_names, case_infos,
    // case_type_0 ..., payload_size and payload_align, then the `which` field and a union of one
    // field per case with the `payload` bytes. Rejects an empty list and clashing names.
    #insert -> string {
        return declare_variant(..types);
    };
}

// Makes a Variant holding `value`; its type picks the case, which must be the only case of that
// type.
//
// ```jai
// #import "Extensions/Variant";
//
// Id :: Variant(s64, string);
// id := variant(Id, "abc");
// assert(id.which == .STRING);
// ```
variant :: ($V: Type, value: $T) -> V {
    CASE :: #run case_of(V, T, "variant");

    result: V;

    #if CASE >= 0 set(*result, value);

    return result;
}

// Stores `value` in the case of type `T` and makes it live, replacing whatever was there without
// releasing it (see free_variant). `T` must be the type of exactly one case.
//
// ```jai
// #import "Extensions/Variant";
//
// Id :: Variant(s64, string);
// id: Id;
// set(*id, "abc");
// set(*id, cast(s64) 7);
// assert(id.which == .S64 && id.s64_ == 7);
// ```
set :: (v: *$V/Variant, value: $T) {
    CASE :: #run case_of(V, T, "set");

    #if CASE >= 0 {
        memset(v.payload.data, 0, V.payload_size);
        v.which = cast(V.Which) (CASE + 1);

        #if size_of(T) > 0 {
            slot := cast(*T) v.payload.data;
            slot.* = value;
        }
    }
}

// A pointer to the `T` case inside the variant, or null when another case is live. It stays
// valid until the variant is set again, freed or moved.
//
// ```jai
// #import "Extensions/Variant";
//
// id := variant(Variant(s64, string), cast(s64) 1);
// number := get_pointer(*id, s64);
// number.* += 1;
// assert(id.s64_ == 2);
// assert(get_pointer(*id, string) == null);
// ```
get_pointer :: (v: *$V/Variant, $T: Type) -> *T {
    CASE :: #run case_of(V, T, "get_pointer");

    if cast(s64) v.which != CASE + 1 return null;

    return cast(*T) v.payload.data;
}

// The same case is live and its values are equal. Strings compare by content, arrays element by
// element, structs member by member; pointers by address. Two NONEs are equal.
//
// ```jai
// #import "Extensions/Variant";
//
// Id :: Variant(s64, string);
// assert(variant(Id, "a") == variant(Id, "a"));
// assert(variant(Id, "a") != variant(Id, cast(s64) 1));
// ```
operator == :: (a: $V/Variant, b: V) -> bool {
    if a.which != b.which return false;
    if a.which == .NONE return true;

    return values_equal(V.case_infos[cast(s64) a.which - 1], a.payload.data, b.payload.data);
}

// The negation of `==`.
operator != :: (a: $V/Variant, b: V) -> bool {
    return !(a == b);
}

// Releases the memory the live case owns, with `free`: strings, resizable arrays (and what their
// elements own), and the same inside structs and nested variants. Views (`[] T`) and pointers are
// not owned and are left alone. Then the variant is NONE. Only use it on cases that came from the
// allocator, not string literals.
//
// ```jai
// #import "Extensions/Variant";
//
// id := variant(Variant(s64, string), copy_string("owned"));
// free_variant(*id);
// assert(id.which == .NONE);
// ```
free_variant :: (v: *$V/Variant) {
    free_variant_value(type_info(V), v);
} @FreesArgument

// ---- printing ----
//
// Basic prints a struct member by member, with no hook for a type of its own. Its one hook is
// `context.print_style.struct_printer`, which Math uses for vectors; this is Variant's.

// A `Print_Style` `struct_printer` that writes a Variant as its live case (`none` for NONE), and passes everything
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
//
// ```jai
// #import "Extensions/Variant";
//
// use_variant_printing();
// id := variant(Variant(s64, string), "abc");
// assert(tprint("%", id) == "abc");
// ```
use_variant_printing :: () {
    style := *context.print_style;

    if style.struct_printer == variant_struct_printer return;

    previous_printer = style.struct_printer;
    previous_data = style.struct_printer_data;
    style.struct_printer = variant_struct_printer;
    style.struct_printer_data = null;
}

// The live case as text (`none` for NONE), allocated with `context.allocator`.
//
// ```jai
// #import "Extensions/Variant";
//
// id := variant(Variant(s64, string), cast(s64) 42);
// assert(variant_to_string(id) == "42");
// ```
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
    found_which := false;

    for (cast(*Type_Info_Struct) info).members {
        if it.name == "case_infos" && (it.flags & .CONSTANT) != 0 found_infos = true;
        if it.name == "which" found_which = true;
    }

    return found_infos && found_which;
}

// The names of the cases of a Variant, in order.
variant_case_names :: (info: *Type_Info) -> [] string {
    for (cast(*Type_Info_Struct) info).members {
        if it.name != "case_names" || it.offset_into_constant_storage < 0 continue;

        storage := (cast(*Type_Info_Struct) info).constant_storage;

        return (cast(*[] string) (storage.data + it.offset_into_constant_storage)).*;
    }

    return .[];
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

#scope_module

previous_printer: (builder: *String_Builder, any: Any, data: *void) -> bool;
previous_data: *void;

append_payload :: (builder: *String_Builder, any: Any) {
    info := cast(*Type_Info_Struct) any.type;
    base := cast(*u8) any.value_pointer;
    cases := variant_case_infos(info);
    payload := offset_of_member(info, "payload");

    index := live_case(info, base);

    if index < 0 {
        append(builder, "none");

        return;
    }

    if index >= cases.count {
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
