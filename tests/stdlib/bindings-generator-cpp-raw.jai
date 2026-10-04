// Bindings_Generator on C++ classes whose tail padding is reused by a derived class (Itanium ABI):
// the base is printed as `X` wrapping a padding-free `X__RAW`, and the derived class embeds the
// RAW variant so its layout matches C++. A library is built with clang++ at compile time and
// called through the generated bindings.
// Skips (printing ok) when clang++ or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#pragma once

namespace Pad {

// Not a POD (user-provided constructor): sizeof is 8 but the data ends after 5 bytes.
struct Base {
    Base();
    int a;
    char b;
    int sum() const;
};

struct Derived : Base {
    char c;
    short d;
    int total() const;
};

struct Longer : Derived {
    char e;
    int all() const;
};

// A chain: Mid is itself the base of a class that reuses its tail padding.
struct Mid : Base {
    char c;
    int mid_sum() const;
};

struct Last : Mid {
    char e;
    int last_sum() const;
};

// POD base: its padding is not reused, no RAW variant.
struct Plain {
    int a;
    char b;
};

struct AfterPlain : Plain {
    char c;
};

Derived* make_derived(int a, int b, int c, int d);
int sizeof_derived();
int offset_of_c();
int offset_of_d();
int offset_of_e();

}
END;

SOURCE :: #string END
#include <cstddef>
#include "pad.h"
namespace Pad {
Base::Base() : a(0), b(0) {}
int Base::sum() const { return a + b; }
int Derived::total() const { return a + b + c + d; }
int Mid::mid_sum() const { return a + b + c; }
int Last::last_sum() const { return a + b + c + e; }
int Longer::all() const { return a + b + c + d + e; }
Derived* make_derived(int a, int b, int c, int d) {
    Derived* r = new Derived();
    r->a = a; r->b = b; r->c = c; r->d = d;
    return r;
}
int sizeof_derived() { return sizeof(Derived); }
int offset_of_c() { return (int)offsetof(Derived, c); }
int offset_of_d() { return (int)offsetof(Derived, d); }
int offset_of_e() { return (int)offsetof(Longer, e); }
}
END;

build_and_generate :: () -> string {
    dir := "/tmp/jai_bindgen_cpp_raw";
    make_directory_if_it_does_not_exist(dir);
    header_path := tprint("%/pad.h", dir);
    source_path := tprint("%/pad.cpp", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libpad.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    result, output := run_command("clang++", "-std=c++17", "-Wno-invalid-offsetof", "-shared", "-fPIC", "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.extra_clang_arguments, "-x", "c++", "-std=c++17");
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libpad");
    opts.generated_library_path_prefix = tprint("%/", dir);
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

byte_offset :: (base: *void, field: *void) -> s64 { return cast(s64) field - cast(s64) base; }

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

main :: () {
    #if HAVE_CPP {
        // The generated shape.
        assert(has(BINDINGS, "Base :: struct {"));
        assert(has(BINDINGS, "using base__raw: Base__RAW;"));
        assert(has(BINDINGS, "Base__RAW :: struct {"));
        assert(has(BINDINGS, "#no_padding"));
        assert(has(BINDINGS, "#as using base__raw: Pad.Base__RAW;"));
        assert(has(BINDINGS, "Mid__RAW :: struct {"));  // Last reuses the tail padding of Mid too
        assert(has(BINDINGS, "#as using mid__raw: Pad.Mid__RAW;"));
        assert(!has(BINDINGS, "Plain__RAW"));
        assert(!has(BINDINGS, "do not use these fields"));

        // Layout matches C++.
        assert(size_of(Pad.Base) == 8);
        assert(size_of(Pad.Base__RAW) == 5);
        assert(size_of(Pad.Derived) == Pad.sizeof_derived());
        assert(size_of(Pad.AfterPlain) == 12);
        assert(size_of(Pad.Mid) == 8 && size_of(Pad.Mid__RAW) == 6 && size_of(Pad.Last) == 8);

        d: Pad.Derived;
        assert(byte_offset(*d, *d.c) == Pad.offset_of_c());
        assert(byte_offset(*d, *d.d) == Pad.offset_of_d());
        l: Pad.Longer;
        assert(byte_offset(*l, *l.e) == Pad.offset_of_e());

        // Fields written by C++ read back in Jai, and the other way round.
        p := Pad.make_derived(1, 2, 3, 4);
        assert(p.a == 1 && p.b == 2 && p.c == 3 && p.d == 4);
        assert(Pad.Derived.total(p) == 10);
        d.a = 10; d.b = 20; d.c = 30; d.d = 40;
        assert(Pad.Derived.total(*d) == 100);

        m: Pad.Last;
        m.a = 1; m.b = 2; m.c = 3; m.e = 4;
        assert(byte_offset(*m, *m.e) == 6);
        assert(Pad.Last.last_sum(*m) == 10);
        assert(Pad.Mid.mid_sum(cast(*Pad.Mid) *m) == 6);

        l.a = 1; l.b = 2; l.c = 3; l.d = 4; l.e = 5;
        assert(Pad.Longer.all(*l) == 15);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
