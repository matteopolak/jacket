#import "Basic";

grow :: (xs: *[..] int) {} @GrowsArray
grow_bytes :: (xs: *[..] u8) {} @GrowsArray
sink :: (p: *int) -> int {
    return p.*;
}

view_sink :: (v: [] int) -> int {
    return v[0];
}

string_sink :: (s: string) -> u8 {
    return s[0];
}

Box :: struct {
    values: [..] int;
    p: *int;
}

Inner :: struct {
    inner: [..] int;
}

grow_outer :: (xs: *[..] Inner) {} @GrowsArray

from_data :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := xs.data;

    grow(*xs);
    sink(p);
}

pointer_copies :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];
    q := p;
    r := q;

    grow(*xs);
    sink(r);
}

local_fields :: () {
    box: Box;

    array_add(*box.values, 1);
    box.p = *box.values[0];
    grow(*box.values);
    sink(box.p);
}

struct_copy :: () {
    xs: [..] int;

    array_add(*xs, 1);
    box: Box;
    box.p = *xs[0];
    other := box;

    grow(*xs);
    sink(other.p);
}

view_copies :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v: [] int = xs;
    copy := v;

    grow(*xs);
    view_sink(copy);
}

string_view :: () {
    xs: [..] u8;

    array_add(*xs, 65);
    s := cast(string) xs;

    grow_bytes(*xs);
    string_sink(s);
}

returned_growth_pointer :: () {
    xs: [..] int;
    p := array_add(*xs);
    xs.count = xs.allocated;
    array_add(*xs, 1);
    sink(p);
}

nested_header :: () {
    outer: [..] Inner;

    array_add(*outer);
    header := *outer[0].inner;

    grow_outer(*outer);
    count := header.count;
}

nested_storage :: () {
    outer: [..] Inner;

    array_add(*outer);
    array_add(*outer[0].inner, 1);
    p := *outer[0].inner[0];

    grow(*outer[0].inner);
    sink(p);
}

explicit_view :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v: [] int;
    v.data = xs.data;
    v.count = 1;
    grow(*xs);
    view_sink(v);
}

string_comparison_reads_storage :: () {
    xs: [..] u8;

    array_add(*xs, 65);
    s := cast(string) xs;

    grow_bytes(*xs);
    same := s == "A";
}

pointer_from_old_view :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v: [] int = xs;

    grow(*xs);
    p := v.data;

    sink(p);
}

invalid_generation_survives_descriptor_replacement :: () {
    xs: [..] int;

    array_add(*xs, 1);
    ys: [..] int;
    p := *xs[0];

    grow(*xs);
    xs = ys;
    grow(*xs);
    sink(p);
}

literal_field :: () {
    xs: [..] int;

    array_add(*xs, 1);
    box := Box.{ p = *xs[0] };

    grow(*xs);
    sink(box.p);
}

literal_view :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v := ([] int).{ data = xs.data, count = 1 };

    grow(*xs);
    view_sink(v);
}

pointer_offset_copy :: () {
    xs: [..] int;

    array_add(*xs, 1);
    p := *xs[0];
    q := *p[0];
    bytes := cast(*u8) q;

    grow(*xs);
    n := bytes.*;
}

direct_view_data_read :: () {
    xs: [..] int;

    array_add(*xs, 1);
    v: [] int = xs;

    grow(*xs);
    n := v.data[0];
}

direct_string_data_read :: () {
    xs: [..] u8;

    array_add(*xs, 65);
    s := cast(string) xs;

    grow_bytes(*xs);
    n := s.data.*;
}

main :: () {}
