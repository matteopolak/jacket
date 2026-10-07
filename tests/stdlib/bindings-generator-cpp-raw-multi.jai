// Bindings_Generator on C++ classes with several base classes where later bases or fields
// reuse the tail padding of an earlier base (Itanium ABI). Each base whose padding is reused is
// printed as a wrapper around a padding-free `X__RAW`, and the derived class (itself RAW when
// it is such a base) spells out the alignment gaps between its bases. A library is built with
// clang++ at compile time; offsets and sizes are compared with what C++ reports.
// Skips (printing ok) when clang++ or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string END
#pragma once

namespace Multi {

struct PartA { PartA(); int a; char b; int sum_a() const; };      // sizeof 8, data 5
struct PartB { PartB(); int x; char y; int sum_b() const; };      // sizeof 8, data 5
struct PartF { PartF(); short f; char g; int sum_f() const; };    // sizeof 4, data 3, alignment 2

// B starts at 8 (alignment), so only B's padding is reused by `c`.
struct C : PartA, PartB { char c; int total() const; };

// F starts at 6, inside A's padding; `z` follows F's data at 9.
struct G : PartA, PartF { char z; int total() const; };

// A derived class that is itself the base of another one: its RAW variant holds the gap
// between its bases.
struct M : PartA, PartF { int m_sum() const; };
struct N : M { char n; int n_sum() const; };

C* make_c(int a, int b, int x, int y, int c);
G* make_g(int a, int b, int f, int g, int z);
N* make_n(int a, int b, int f, int g, int n);
int sizeof_c();
int sizeof_g();
int sizeof_m();
int sizeof_n();
int offset_b_in_c();
int offset_c();
int offset_f_in_g();
int offset_z();
int offset_n();

}
END;

SOURCE :: #string END
#include <cstddef>
#include "multi.h"
namespace Multi {
PartA::PartA() : a(0), b(0) {}
PartB::PartB() : x(0), y(0) {}
PartF::PartF() : f(0), g(0) {}
int PartA::sum_a() const { return a + b; }
int PartB::sum_b() const { return x + y; }
int PartF::sum_f() const { return f + g; }
int C::total() const { return a + b + x + y + c; }
int G::total() const { return a + b + f + g + z; }
int M::m_sum() const { return a + b + f + g; }
int N::n_sum() const { return a + b + f + g + n; }
C* make_c(int a, int b, int x, int y, int c) {
    C* r = new C();
    r->a = a; r->b = b; r->x = x; r->y = y; r->c = c;
    return r;
}
G* make_g(int a, int b, int f, int g, int z) {
    G* r = new G();
    r->a = a; r->b = b; r->f = f; r->g = g; r->z = z;
    return r;
}
N* make_n(int a, int b, int f, int g, int n) {
    N* r = new N();
    r->a = a; r->b = b; r->f = f; r->g = g; r->n = n;
    return r;
}
int sizeof_c() { return sizeof(C); }
int sizeof_g() { return sizeof(G); }
int sizeof_m() { return sizeof(M); }
int sizeof_n() { return sizeof(N); }
int offset_b_in_c() { C c; return (int)((char*)static_cast<PartB*>(&c) - (char*)&c); }
int offset_c() { return (int)offsetof(C, c); }
int offset_f_in_g() { G g; return (int)((char*)static_cast<PartF*>(&g) - (char*)&g); }
int offset_z() { return (int)offsetof(G, z); }
int offset_n() { return (int)offsetof(N, n); }
}
END;

build_and_generate :: () -> string {
    // The WASM target (the browser playground) starts no processes and loads no native
    // libraries, so there is no compiler or libclang to call: skip like a host without them.
    #if OS == .WASM return "";
    // Kept: the program loads the library from here when it runs.
    dir := scratch_directory("bindgen-cpp-raw-multi");
    header_path := tprint("%/multi.h", dir);
    source_path := tprint("%/multi.cpp", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libmulti.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    result := run_command("clang++", "-std=c++17", "-Wno-invalid-offsetof", "-shared", "-fPIC", "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.extra_clang_arguments, "-x", "c++", "-std=c++17");
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libmulti");
    opts.generate_compile_time_struct_checks = true;
    builder, ok := generate_bindings(opts);
    if !ok return "";
    return builder_to_string(*builder);
}

BINDINGS :: #run build_and_generate();
HAVE_CPP :: BINDINGS.count > 0;

#if HAVE_CPP {
    #insert BINDINGS;
}

byte_offset :: (base: *void, field: *void) -> s64 {
    return cast(s64) field - cast(s64) base;
}

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

main :: () {
    #if HAVE_CPP {
        // The generated shape: PartB, PartF, PartA (G puts F inside A's padding) and M are reused.
        assert(has(BINDINGS, "PartB__RAW :: struct {"));
        assert(has(BINDINGS, "PartF__RAW :: struct {"));
        assert(has(BINDINGS, "PartA__RAW :: struct {"));
        assert(has(BINDINGS, "M__RAW :: struct {"));
        assert(!has(BINDINGS, "do not use these fields"));

        assert(size_of(Multi.C) == Multi.sizeof_c());
        assert(size_of(Multi.G) == Multi.sizeof_g());
        assert(size_of(Multi.M) == Multi.sizeof_m());
        assert(size_of(Multi.N) == Multi.sizeof_n());
        assert(size_of(Multi.PartB__RAW) == 5 && size_of(Multi.PartF__RAW) == 3);

        c: Multi.C;
        assert(byte_offset(*c, *c.x) == Multi.offset_b_in_c());
        assert(byte_offset(*c, *c.c) == Multi.offset_c());
        g: Multi.G;
        assert(byte_offset(*g, *g.f) == Multi.offset_f_in_g());
        assert(byte_offset(*g, *g.z) == Multi.offset_z());
        n: Multi.N;
        assert(byte_offset(*n, *n.n) == Multi.offset_n());

        // Fields written by C++ read back in Jai, and the other way round.
        pc := Multi.make_c(1, 2, 3, 4, 5);
        assert(pc.a == 1 && pc.b == 2 && pc.x == 3 && pc.y == 4 && pc.c == 5);
        assert(Multi.C.total(pc) == 15);
        c.a = 10;
        c.b = 20;
        c.x = 30;
        c.y = 40;
        c.c = 50;
        assert(Multi.C.total(*c) == 150);

        pg := Multi.make_g(1, 2, 3, 4, 5);
        assert(pg.a == 1 && pg.b == 2 && pg.f == 3 && pg.g == 4 && pg.z == 5);
        assert(Multi.G.total(pg) == 15);
        g.a = 10;
        g.b = 20;
        g.f = 30;
        g.g = 40;
        g.z = 50;
        assert(Multi.G.total(*g) == 150);

        pn := Multi.make_n(1, 2, 3, 4, 5);
        assert(pn.a == 1 && pn.b == 2 && pn.f == 3 && pn.g == 4 && pn.n == 5);
        n.a = 10;
        n.b = 20;
        n.f = 30;
        n.g = 40;
        n.n = 50;
        assert(Multi.N.n_sum(*n) == 150);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
