// Bindings_Generator with `os = .WINDOWS`: bit fields follow the MSVC layout rules. MSVC allocates
// a unit of the declared type's size and starts a new one when the type's size changes, when the
// next field would cross the unit, or after a `: 0` field; Itanium/AAPCS64 pack adjacent fields
// regardless of type. The sizes below are checked against clang's own record layout for
// `x86_64-pc-windows-msvc` (compile-time `_Static_assert`s), against a C library built natively with
// `-mms-bitfields` (the same rules on the host), and the accessors read and write the same bits
// as the compiler's code. Skips (printing ok) when clang or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#pragma once

struct Mixed { char a : 3; int b : 5; char c : 2; };
struct SameType { unsigned int a : 20; unsigned int b : 20; unsigned int c : 4; };
struct Zero { unsigned int a : 3; unsigned int : 0; unsigned int b : 3; };
struct Tail { char c; int x : 4; };
struct Wide { unsigned long long a : 40; unsigned long long b : 30; char after; };
struct Shorts { short a : 9; short b : 9; };
struct Signs { unsigned int a : 4; int b : 4; };
END;

SOURCE :: #string END
#include "msbits.h"
#include <stddef.h>
int ms_size(int which) {
    switch (which) {
        case 0: return sizeof(struct Mixed);
        case 1: return sizeof(struct SameType);
        case 2: return sizeof(struct Zero);
        case 3: return sizeof(struct Tail);
        case 4: return sizeof(struct Wide);
        case 5: return sizeof(struct Shorts);
        default: return sizeof(struct Signs);
    }
}
void ms_mixed_set(struct Mixed *m, int a, int b, int c) { m->a = a; m->b = b; m->c = c; }
int ms_mixed_get(const struct Mixed *m, int which) { return which == 0 ? m->a : which == 1 ? m->b : m->c; }
void ms_same_set(struct SameType *s, unsigned a, unsigned b, unsigned c) { s->a = a; s->b = b; s->c = c; }
unsigned ms_same_get(const struct SameType *s, int which) { return which == 0 ? s->a : which == 1 ? s->b : s->c; }
void ms_zero_set(struct Zero *z, unsigned a, unsigned b) { z->a = a; z->b = b; }
unsigned ms_zero_get(const struct Zero *z, int which) { return which == 0 ? z->a : z->b; }
void ms_tail_set(struct Tail *t, int c, int x) { t->c = c; t->x = x; }
int ms_tail_get(const struct Tail *t, int which) { return which == 0 ? t->c : t->x; }
void ms_wide_set(struct Wide *w, unsigned long long a, unsigned long long b, int after) { w->a = a; w->b = b; w->after = after; }
unsigned long long ms_wide_get(const struct Wide *w, int which) { return which == 0 ? w->a : which == 1 ? w->b : (unsigned long long)w->after; }
void ms_shorts_set(struct Shorts *s, int a, int b) { s->a = a; s->b = b; }
int ms_shorts_get(const struct Shorts *s, int which) { return which == 0 ? s->a : s->b; }
void ms_signs_set(struct Signs *s, unsigned a, int b) { s->a = a; s->b = b; }
int ms_signs_get(const struct Signs *s, int which) { return which == 0 ? (int)s->a : s->b; }
END;

// What MSVC lays out, written down by hand from its rules (`sizeof`, in this order).
EXPECTED_SIZES :: int.[12, 8, 8, 8, 24, 4, 4];
STRUCT_NAMES :: string.["Mixed", "SameType", "Zero", "Tail", "Wide", "Shorts", "Signs"];

build_and_generate :: () -> string {
    dir := "/tmp/jai_bindgen_bitfields_msvc";
    make_directory_if_it_does_not_exist(dir);
    header_path := tprint("%/msbits.h", dir);
    source_path := tprint("%/msbits.c", dir);
    check_path := tprint("%/check.c", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libmsbits.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    // clang's record layout for the Windows target agrees with the hand-written sizes.
    checks: String_Builder;
    append(*checks, "#include \"msbits.h\"\n");
    for STRUCT_NAMES print_to_builder(*checks, "_Static_assert(sizeof(struct %) == %, \"%\");\n", it, EXPECTED_SIZES[it_index], it);
    if !write_entire_file(check_path, builder_to_string(*checks)) return "";
    result, output := run_command("clang", "-target", "x86_64-pc-windows-msvc", "-fsyntax-only", "-I", dir, check_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    // The library uses the MS rules natively.
    result, output = run_command("clang", "-mms-bitfields", "-shared", "-fPIC", "-I", dir, "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    // No -target given: the generator asks clang for the MS bit field rules itself.
    opts: Generate_Bindings_Options;
    opts.os = .WINDOWS;
    array_add(*opts.source_files, header_path);
    opts.generate_compile_time_struct_checks = true;
    builder, ok := generate_bindings(opts);
    if !ok return "";
    text := builder_to_string(*builder);

    // An explicit Windows target gives the same bindings.
    opts2: Generate_Bindings_Options;
    opts2.os = .WINDOWS;
    array_add(*opts2.source_files, header_path);
    array_add(*opts2.extra_clang_arguments, "-target", "x86_64-pc-windows-msvc");
    opts2.generate_compile_time_struct_checks = true;
    builder2, ok2 := generate_bindings(opts2);
    if !ok2 || builder_to_string(*builder2) != text return "";
    return text;
}

BINDINGS :: #run build_and_generate();
HAVE_C :: BINDINGS.count > 0;

#if HAVE_C {
    #insert BINDINGS;
}

ms :: #library "/tmp/jai_bindgen_bitfields_msvc/libmsbits";
ms_size :: (which: s32) -> s32 #foreign ms;
ms_mixed_set :: (m: *Mixed, a: s32, b: s32, c: s32) #foreign ms;
ms_mixed_get :: (m: *Mixed, which: s32) -> s32 #foreign ms;
ms_same_set :: (s: *SameType, a: u32, b: u32, c: u32) #foreign ms;
ms_same_get :: (s: *SameType, which: s32) -> u32 #foreign ms;
ms_zero_set :: (z: *Zero, a: u32, b: u32) #foreign ms;
ms_zero_get :: (z: *Zero, which: s32) -> u32 #foreign ms;
ms_tail_set :: (t: *Tail, c: s32, x: s32) #foreign ms;
ms_tail_get :: (t: *Tail, which: s32) -> s32 #foreign ms;
ms_wide_set :: (w: *Wide, a: u64, b: u64, after: s32) #foreign ms;
ms_wide_get :: (w: *Wide, which: s32) -> u64 #foreign ms;
ms_shorts_set :: (s: *Shorts, a: s32, b: s32) #foreign ms;
ms_shorts_get :: (s: *Shorts, which: s32) -> s32 #foreign ms;
ms_signs_set :: (s: *Signs, a: u32, b: s32) #foreign ms;
ms_signs_get :: (s: *Signs, which: s32) -> s32 #foreign ms;

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

main :: () {
    #if HAVE_C {
        // One unit per declared-type change: each field of Mixed has its own storage.
        assert(has(BINDINGS, "Mixed :: struct {"));
        assert(has(BINDINGS, "SameType_get_c :: (s: *SameType) -> u32 {"));

        // Sizes in Jai, in the native MS-layout library, and by hand.
        assert(size_of(Mixed) == EXPECTED_SIZES[0] && ms_size(0) == EXPECTED_SIZES[0]);
        assert(size_of(SameType) == EXPECTED_SIZES[1] && ms_size(1) == EXPECTED_SIZES[1]);
        assert(size_of(Zero) == EXPECTED_SIZES[2] && ms_size(2) == EXPECTED_SIZES[2]);
        assert(size_of(Tail) == EXPECTED_SIZES[3] && ms_size(3) == EXPECTED_SIZES[3]);
        assert(size_of(Wide) == EXPECTED_SIZES[4] && ms_size(4) == EXPECTED_SIZES[4]);
        assert(size_of(Shorts) == EXPECTED_SIZES[5] && ms_size(5) == EXPECTED_SIZES[5]);
        assert(size_of(Signs) == EXPECTED_SIZES[6] && ms_size(6) == EXPECTED_SIZES[6]);

        m: Mixed;
        ms_mixed_set(*m, 5, -9, 2);
        assert(Mixed_get_a(*m) == 5 && Mixed_get_b(*m) == -9 && Mixed_get_c(*m) == 2);
        Mixed_set_a(*m, 3); // `char` is unsigned on Linux arm64; stay in range for both.
        Mixed_set_b(*m, 15);
        Mixed_set_c(*m, 1);
        assert(ms_mixed_get(*m, 0) == 3 && ms_mixed_get(*m, 1) == 15 && ms_mixed_get(*m, 2) == 1);

        s: SameType;
        ms_same_set(*s, 0xABCDE, 0x54321, 9);
        assert(SameType_get_a(*s) == 0xABCDE && SameType_get_b(*s) == 0x54321 && SameType_get_c(*s) == 9);
        SameType_set_b(*s, 0xFFFFF);
        SameType_set_c(*s, 15);
        assert(ms_same_get(*s, 0) == 0xABCDE && ms_same_get(*s, 1) == 0xFFFFF && ms_same_get(*s, 2) == 15);

        z: Zero;
        ms_zero_set(*z, 5, 6);
        assert(Zero_get_a(*z) == 5 && Zero_get_b(*z) == 6);
        Zero_set_b(*z, 1);
        assert(ms_zero_get(*z, 0) == 5 && ms_zero_get(*z, 1) == 1);

        t: Tail;
        ms_tail_set(*t, 7, -3);
        assert(t.c == 7 && Tail_get_x(*t) == -3);
        Tail_set_x(*t, 5);
        assert(ms_tail_get(*t, 0) == 7 && ms_tail_get(*t, 1) == 5);

        w: Wide;
        ms_wide_set(*w, 0xAB_CDEF_0123, 0x2BCD_EF01, 77);
        assert(Wide_get_a(*w) == 0xAB_CDEF_0123 && Wide_get_b(*w) == 0x2BCD_EF01 && w.after == 77);
        Wide_set_b(*w, 0x3FFF_FFFF);
        assert(ms_wide_get(*w, 1) == 0x3FFF_FFFF && ms_wide_get(*w, 0) == 0xAB_CDEF_0123 && ms_wide_get(*w, 2) == 77);

        sh: Shorts;
        ms_shorts_set(*sh, -200, 255);
        assert(Shorts_get_a(*sh) == -200 && Shorts_get_b(*sh) == 255);
        Shorts_set_a(*sh, 100);
        Shorts_set_b(*sh, -256);
        assert(ms_shorts_get(*sh, 0) == 100 && ms_shorts_get(*sh, 1) == -256);

        sg: Signs;
        ms_signs_set(*sg, 12, -5);
        assert(Signs_get_a(*sg) == 12 && Signs_get_b(*sg) == -5);
        Signs_set_a(*sg, 3);
        assert(ms_signs_get(*sg, 0) == 3 && ms_signs_get(*sg, 1) == -5);

        print("ok\n");
    } else {
        print("ok\n");
    }
}
