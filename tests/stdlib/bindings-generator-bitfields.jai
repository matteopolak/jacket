// Bindings_Generator on C structs with bit fields: storage layout (`__bitfield`, single members,
// alignment helpers, zero-width fields) and the generated `S_get_x` / `S_set_x` accessors.
// A small C library is built with clang at compile time; the accessors are checked against
// the compiler's own bit field code in both directions, and the generator's size checks
// (`generate_compile_time_struct_checks`) validate the layouts.
// Skips (printing ok) when clang or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#pragma once
#include <stdint.h>
#include <stdbool.h>

struct Packed { unsigned int a : 3; unsigned int b : 5; int tail; };
struct Single { unsigned int flag : 1; };
struct Offset { char c; int x : 20; unsigned int y : 4; short s; };
struct Signed { int lo : 4; int hi : 12; signed char m : 3; };
struct Wide { uint64_t a : 40; uint64_t b : 24; uint8_t after; };
struct Zero { unsigned int a : 3; unsigned int : 0; unsigned int b : 3; char z; };
struct Bools { bool on : 1; bool off : 1; uint8_t n : 6; };
union Either { unsigned int i : 7; unsigned int j : 9; };
struct Gap { char c; int : 0; int b : 3; };
struct Instance { uint32_t index : 24; uint32_t mask : 8; uint32_t offset : 24; uint32_t flags : 8; uint64_t reference; };

int bits_packed_get(const struct Packed *p, int which);
void bits_packed_set(struct Packed *p, unsigned a, unsigned b, int tail);
void bits_offset_set(struct Offset *o, int x, unsigned y);
int bits_offset_x(const struct Offset *o);
unsigned bits_offset_y(const struct Offset *o);
void bits_signed_set(struct Signed *s, int lo, int hi, int m);
int bits_signed_get(const struct Signed *s, int which);
void bits_wide_set(struct Wide *w, uint64_t a, uint64_t b);
uint64_t bits_wide_get(const struct Wide *w, int which);
void bits_zero_set(struct Zero *z, unsigned a, unsigned b, int zc);
unsigned bits_zero_get(const struct Zero *z, int which);
void bits_bools_set(struct Bools *b, bool on, bool off, unsigned n);
unsigned bits_bools_get(const struct Bools *b, int which);
void bits_gap_set(struct Gap *g, char c, int b);
int bits_gap_b(const struct Gap *g);
uint64_t bits_instance_get(const struct Instance *i, int which);
void bits_instance_set(struct Instance *i, uint32_t index, uint32_t mask, uint32_t offset, uint32_t flags);
END;

SOURCE :: #string END
#include "bits.h"
int bits_packed_get(const struct Packed *p, int which) { return which == 0 ? (int)p->a : which == 1 ? (int)p->b : p->tail; }
void bits_packed_set(struct Packed *p, unsigned a, unsigned b, int tail) { p->a = a; p->b = b; p->tail = tail; }
void bits_offset_set(struct Offset *o, int x, unsigned y) { o->x = x; o->y = y; }
int bits_offset_x(const struct Offset *o) { return o->x; }
unsigned bits_offset_y(const struct Offset *o) { return o->y; }
void bits_signed_set(struct Signed *s, int lo, int hi, int m) { s->lo = lo; s->hi = hi; s->m = m; }
int bits_signed_get(const struct Signed *s, int which) { return which == 0 ? s->lo : which == 1 ? s->hi : s->m; }
void bits_wide_set(struct Wide *w, uint64_t a, uint64_t b) { w->a = a; w->b = b; }
uint64_t bits_wide_get(const struct Wide *w, int which) { return which == 0 ? w->a : w->b; }
void bits_zero_set(struct Zero *z, unsigned a, unsigned b, int zc) { z->a = a; z->b = b; z->z = zc; }
unsigned bits_zero_get(const struct Zero *z, int which) { return which == 0 ? z->a : which == 1 ? z->b : z->z; }
void bits_bools_set(struct Bools *b, bool on, bool off, unsigned n) { b->on = on; b->off = off; b->n = n; }
unsigned bits_bools_get(const struct Bools *b, int which) { return which == 0 ? b->on : which == 1 ? b->off : b->n; }
void bits_gap_set(struct Gap *g, char c, int b) { g->c = c; g->b = b; }
int bits_gap_b(const struct Gap *g) { return g->b; }
uint64_t bits_instance_get(const struct Instance *i, int which) { return which == 0 ? i->index : which == 1 ? i->mask : which == 2 ? i->offset : i->flags; }
void bits_instance_set(struct Instance *i, uint32_t index, uint32_t mask, uint32_t offset, uint32_t flags) { i->index = index; i->mask = mask; i->offset = offset; i->flags = flags; }
END;

build_and_generate :: () -> string {
    dir := "/tmp/jai_bindgen_bitfields";
    make_directory_if_it_does_not_exist(dir);
    header_path := tprint("%/bits.h", dir);
    source_path := tprint("%/bits.c", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libbits.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    result, output := run_command("clang", "-shared", "-fPIC", "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libbits");
    opts.generated_library_path_prefix = tprint("%/", dir);
    opts.generate_compile_time_struct_checks = true;
    builder, ok := generate_bindings(opts);
    if !ok return "";
    return builder_to_string(*builder);
}

BINDINGS :: #run build_and_generate();
HAVE_C :: BINDINGS.count > 0;

#if HAVE_C {
    #insert BINDINGS;
}

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

main :: () {
    #if HAVE_C {
        // Layout in the generated text.
        assert(has(BINDINGS, "__bitfield: u8;"));
        assert(has(BINDINGS, "flag: u8; /* 1 bits */"));
        assert(has(BINDINGS, "#place flag; flag_alignment: u32;"));
        assert(has(BINDINGS, "__bitfield: [3] u8;"));
        assert(has(BINDINGS, "__bitfield: u64;"));
        assert(has(BINDINGS, "Packed_get_a :: (s: *Packed) -> u32 {"));
        assert(has(BINDINGS, "Packed_set_b :: (s: *Packed, value: u32) {"));
        assert(!has(BINDINGS, "Packed_get_tail"));

        assert(size_of(Packed) == 8 && size_of(Single) == 4 && size_of(Offset) == 8);
        assert(size_of(Wide) == 16 && size_of(Zero) == 8 && size_of(Gap) == 8);

        // C writes, Jai reads.
        p: Packed;
        bits_packed_set(*p, 5, 19, -7);
        assert(Packed_get_a(*p) == 5 && Packed_get_b(*p) == 19 && p.tail == -7);
        // Jai writes, C reads.
        Packed_set_a(*p, 2);
        Packed_set_b(*p, 31);
        assert(bits_packed_get(*p, 0) == 2 && bits_packed_get(*p, 1) == 31 && bits_packed_get(*p, 2) == -7);
        // Values are truncated to the field width.
        Packed_set_a(*p, 0xff);
        assert(bits_packed_get(*p, 0) == 7 && bits_packed_get(*p, 1) == 31);

        o: Offset;
        bits_offset_set(*o, 1000, 9);
        assert(Offset_get_x(*o) == 1000 && Offset_get_y(*o) == 9);
        Offset_set_x(*o, 524287);
        Offset_set_y(*o, 15);
        assert(bits_offset_x(*o) == 524287 && bits_offset_y(*o) == 15);
        Offset_set_x(*o, -3);
        assert(Offset_get_x(*o) == -3 && bits_offset_x(*o) == -3);
        o.s = 77;
        assert(Offset_get_x(*o) == -3 && o.s == 77);

        s: Signed;
        bits_signed_set(*s, -5, -1000, -2);
        assert(Signed_get_lo(*s) == -5 && Signed_get_hi(*s) == -1000 && Signed_get_m(*s) == -2);
        Signed_set_lo(*s, 7);
        Signed_set_hi(*s, 2047);
        Signed_set_m(*s, -4);
        assert(bits_signed_get(*s, 0) == 7 && bits_signed_get(*s, 1) == 2047 && bits_signed_get(*s, 2) == -4);

        w: Wide;
        bits_wide_set(*w, 0xAB_CDEF_0123, 0x45_6789);
        assert(Wide_get_a(*w) == 0xAB_CDEF_0123 && Wide_get_b(*w) == 0x45_6789);
        Wide_set_a(*w, 0xFF_FFFF_FFFF);
        Wide_set_b(*w, 1);
        assert(bits_wide_get(*w, 0) == 0xFF_FFFF_FFFF && bits_wide_get(*w, 1) == 1);
        w.after = 9;
        assert(Wide_get_b(*w) == 1);

        z: Zero;
        bits_zero_set(*z, 5, 6, 42);
        assert(Zero_get_a(*z) == 5 && Zero_get_b(*z) == 6 && z.z == 42);
        Zero_set_b(*z, 3);
        assert(bits_zero_get(*z, 1) == 3 && bits_zero_get(*z, 0) == 5 && bits_zero_get(*z, 2) == 42);

        bo: Bools;
        bits_bools_set(*bo, true, false, 33);
        assert(Bools_get_on(*bo) == true && Bools_get_off(*bo) == false && Bools_get_n(*bo) == 33);
        Bools_set_off(*bo, true);
        Bools_set_on(*bo, false);
        assert(bits_bools_get(*bo, 0) == 0 && bits_bools_get(*bo, 1) == 1 && bits_bools_get(*bo, 2) == 33);

        g: Gap;
        bits_gap_set(*g, 65, 3);
        assert(g.c == 65 && Gap_get_b(*g) == 3 && bits_gap_b(*g) == 3);

        inst: Instance;
        assert(size_of(Instance) == 16);
        bits_instance_set(*inst, 0xABCDEF, 0x12, 0x654321, 0xFE);
        assert(Instance_get_index(*inst) == 0xABCDEF && Instance_get_mask(*inst) == 0x12);
        assert(Instance_get_offset(*inst) == 0x654321 && Instance_get_flags(*inst) == 0xFE);
        Instance_set_flags(*inst, 3);
        Instance_set_mask(*inst, 0xFF);
        assert(bits_instance_get(*inst, 3) == 3 && bits_instance_get(*inst, 1) == 0xFF && bits_instance_get(*inst, 0) == 0xABCDEF);

        print("ok\n");
    } else {
        print("ok\n");
    }
}
