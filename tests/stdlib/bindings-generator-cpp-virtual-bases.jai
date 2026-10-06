// Bindings_Generator on C++ classes with virtual bases: the class gets its own vtable pointer
// when nothing else provides one, the virtual base follows the class's members at the offset
// clang computes, and overloads that look the same in Jai are renamed and commented.
// A small library is built with clang++ at compile time; the program reads objects it creates.
// Skips (printing ok) when clang++ or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
struct Base { int b; double d; };
struct Small { int s; };
struct Left : virtual Base { int l; };
struct Right : virtual Base { virtual int f(); int r; };
struct Tight : virtual Small { char c; };
struct Diamond : Left, Right { int x; };

extern "C" Left *make_left(int b, double d, int l);
extern "C" Right *make_right(int b, int r);
extern "C" Tight *make_tight(int s, char c);
extern "C" int left_sum(Left *p);

void take(int &v);
void take(const int &v);
END;

SOURCE :: #string END
#include "vbases.h"
int Right::f() { return r * 2; }
extern "C" Left *make_left(int b, double d, int l) { Left *p = new Left(); p->b = b; p->d = d; p->l = l; return p; }
extern "C" Right *make_right(int b, int r) { Right *p = new Right(); p->b = b; p->r = r; return p; }
extern "C" Tight *make_tight(int s, char c) { Tight *p = new Tight(); p->s = s; p->c = c; return p; }
extern "C" int left_sum(Left *p) { return p->b + p->l + (int) p->d; }
void take(int &v) { v += 1; }
void take(const int &v) {}
END;

build_and_generate :: () -> string {
    dir := "/tmp/jai_bindgen_cpp_vbases";
    make_directory_if_it_does_not_exist(dir);
    header_path := tprint("%/vbases.h", dir);
    source_path := tprint("%/vbases.cpp", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libvbases.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    result := run_command("clang++", "-std=c++17", "-shared", "-fPIC", "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.extra_clang_arguments, "-x", "c++", "-std=c++17");
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libvbases");
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

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

main :: () {
    #if HAVE_CPP {
        assert(has(BINDINGS, "__vptr: *void; // C++ virtual bases make the class dynamic"));
        assert(has(BINDINGS, "using base: Base; // C++ virtual base"));
        assert(has(BINDINGS, "using small: Small #align 4; // C++ virtual base") || has(BINDINGS, "using small: Small; // C++ virtual base"));
        // libclang before 20 cannot report virtual base offsets (jaic prefers a newer one when
        // several are installed); the generator then says so instead of checking the offset.
        assert(has(BINDINGS, "(virtual base) has unexpected offset") || has(BINDINGS, "// C++ virtual base (offset unknown to this libclang)"));
        assert(has(BINDINGS, "// jai: a base class has C++ virtual bases"));
        assert(has(BINDINGS, "take :: (v: /*reference*/ *s32) -> void"));
        assert(has(BINDINGS, "take_1 :: (v: /*const reference*/ *s32) -> void"));

        // Layouts match clang's (the generated #run checks also assert them).
        assert(size_of(Left) == 32);
        assert(size_of(Tight) == 16);

        left := make_left(3, 4.0, 5);
        assert(left.b == 3 && left.d == 4.0 && left.l == 5);
        assert(left_sum(left) == 12);
        left.l = 10;
        assert(left_sum(left) == 17);

        right := make_right(7, 21);
        assert(right.b == 7 && right.r == 21);
        assert(Right_f(right) == 42);

        tight := make_tight(9, #char "z");
        assert(tight.s == 9 && tight.c == #char "z");

        value: s32 = 1;
        take(*value);
        assert(value == 2);
    }
    print("ok\n");
}
