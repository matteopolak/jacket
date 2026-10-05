// Bindings_Generator on a C++ library: member functions, constructors and destructors,
// virtual functions through a vtable, inheritance, static members, operators, class templates
// by-value struct passing and printf-like wrappers. A small library is built with clang++ at
// compile time, bindings
// are generated for its header (`#insert`ed into this file), and the program calls into it.
// Skips (printing ok) when clang++ or libclang is unavailable.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#pragma once
#include <stdint.h>

namespace Geo {

struct Vec2 {
    float x, y;
    Vec2() : x(0), y(0) {}
    Vec2(float x_, float y_) : x(x_), y(y_) {}
    float length_sq() const;
    void scale(float f);
    Vec2 operator+(const Vec2& o) const;
    Vec2 operator-(const Vec2& o) const;
    bool operator==(const Vec2& o) const;
    float& operator[](int i);
    static Vec2 make(float v);
    static int instances;
    static const int MAX_THINGS = 16;
};

struct Counter {
    int value;
    Counter();
    Counter(int start);
    Counter(const Counter& other);
    ~Counter();
    void add(int n);
    int get() const;
    static int live;
};

class Shape {
public:
    int id;
    int tag = 0;
    Shape(int id_);
    virtual ~Shape();
    virtual float area() const;
    virtual const char* name() const;
    virtual void set_id(int i);
    int get_id() const;
};

class Circle : public Shape {
public:
    float radius;
    Circle(int id_, float r);
    ~Circle() override;
    float area() const override;
    const char* name() const override;
    virtual float circumference() const;
};

struct Pod { int a; int b; };

struct Base1 { int b1; int get_b1() const; };
struct Derived1 : public Base1 { int d1; int sum() const; };

template <typename T>
struct Box {
    T value;
    int count;
};

struct Holder {
    Box<int> ibox;
    Box<float>* fbox;
    Pod pod;
};

struct Big { int v[8]; };
Big make_big(int x);
int sum_big(Big b);
struct Mixed { int a; float b; };
Mixed make_mixed(int a, float b);
float mixed_sum(Mixed m);
Counter make_counter(int v);
Vec2 add_vec(const Vec2& a, const Vec2& b);
int free_function(int a, int b = 3);

} // namespace Geo

extern "C" int shapes_live_counters();
extern "C" Geo::Shape* shapes_new_circle(int id, float r);
extern "C" void shapes_delete(Geo::Shape* s);
extern "C" int shapes_format(char* out, int capacity, const char* fmt, ...);
END;

SOURCE :: #string END
#include "shapes.h"
namespace Geo {
int Vec2::instances = 0;
float Vec2::length_sq() const { return x*x + y*y; }
void Vec2::scale(float f) { x *= f; y *= f; }
Vec2 Vec2::operator+(const Vec2& o) const { return Vec2(x+o.x, y+o.y); }
Vec2 Vec2::operator-(const Vec2& o) const { return Vec2(x-o.x, y-o.y); }
bool Vec2::operator==(const Vec2& o) const { return x==o.x && y==o.y; }
float& Vec2::operator[](int i) { return i == 0 ? x : y; }
Vec2 Vec2::make(float v) { instances++; return Vec2(v, v); }

int Counter::live = 0;
Counter::Counter() : value(0) { live++; }
Counter::Counter(int s) : value(s) { live++; }
Counter::Counter(const Counter& o) : value(o.value) { live++; }
Counter::~Counter() { live--; }
void Counter::add(int n) { value += n; }
int Counter::get() const { return value; }

Shape::Shape(int i) : id(i) {}
Shape::~Shape() {}
float Shape::area() const { return 0; }
const char* Shape::name() const { return "shape"; }
void Shape::set_id(int i) { id = i; }
int Shape::get_id() const { return id; }

Circle::Circle(int i, float r) : Shape(i), radius(r) {}
Circle::~Circle() {}
float Circle::area() const { return 3.0f * radius * radius; }
const char* Circle::name() const { return "circle"; }
float Circle::circumference() const { return 6.0f * radius; }

int Base1::get_b1() const { return b1; }
int Derived1::sum() const { return b1 + d1; }

Big make_big(int x) { Big b; for (int i = 0; i < 8; i++) b.v[i] = x + i; return b; }
int sum_big(Big b) { int s = 0; for (int i = 0; i < 8; i++) s += b.v[i]; return s; }
Mixed make_mixed(int a, float b) { Mixed m; m.a = a; m.b = b; return m; }
float mixed_sum(Mixed m) { return m.a + m.b; }
Counter make_counter(int v) { return Counter(v); }
Vec2 add_vec(const Vec2& a, const Vec2& b) { return a + b; }
int free_function(int a, int b) { return a * b; }
}
extern "C" int shapes_live_counters() { return Geo::Counter::live; }
extern "C" Geo::Shape* shapes_new_circle(int id, float r) { return new Geo::Circle(id, r); }
extern "C" void shapes_delete(Geo::Shape* s) { delete s; }
#include <stdarg.h>
#include <stdio.h>
extern "C" int shapes_format(char* out, int capacity, const char* fmt, ...) {
    va_list args;
    va_start(args, fmt);
    int n = vsnprintf(out, capacity, fmt, args);
    va_end(args);
    return n;
}
END;

build_and_generate :: () -> string {
    dir := "/tmp/jai_bindgen_cpp_classes";
    make_directory_if_it_does_not_exist(dir);
    header_path := tprint("%/shapes.h", dir);
    source_path := tprint("%/shapes.cpp", dir);
    ext := ifx OS == .MACOS then "dylib" else "so";
    library_path := tprint("%/libshapes.%", dir, ext);
    if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

    result, output := run_command("clang++", "-std=c++17", "-shared", "-fPIC", "-o", library_path, source_path, capture_and_return_output = true);
    if result.exit_code != 0 return "";

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.extra_clang_arguments, "-x", "c++", "-std=c++17");
    array_add(*opts.libpaths, dir);
    array_add(*opts.libnames, "libshapes");
    opts.generate_compile_time_struct_checks = true;
    builder, ok := generate_bindings(opts);
    if !ok return "";
    return builder_to_string(*builder);
}

BINDINGS :: #run build_and_generate();
HAVE_CPP :: BINDINGS.count > 0;

#if HAVE_CPP {
    #insert BINDINGS;

    live :: () -> s32 #foreign libshapes "shapes_live_counters";
    new_circle :: (id: s32, r: float) -> *Geo.Shape #foreign libshapes "shapes_new_circle";
    delete_shape :: (s: *Geo.Shape) #foreign libshapes "shapes_delete";
}

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

main :: () {
    #if HAVE_CPP {
        // The generated declarations.
        assert(has(BINDINGS, "Constructor :: (this: *Geo.Counter, start: s32) -> void #cpp_method #foreign libshapes"));
        assert(has(BINDINGS, "Destructor_Base :: (this: *Geo.Counter)"));
        assert(has(BINDINGS, "#as using shape: Geo.Shape;"));
        assert(has(BINDINGS, "Shape_VTable :: struct #type_info_none {"));
        assert(has(BINDINGS, "Shape_area :: inline (this: *Geo.Shape) -> float { return this.vtable.area(this); }"));
        assert(has(BINDINGS, "#place shape; circle_vtable: *Circle_VTable;"));
        assert(has(BINDINGS, "Box :: struct(T: Type) {"));
        assert(has(BINDINGS, "ibox: Geo.Box(s32);"));
        assert(has(BINDINGS, "-> Geo.Counter #cpp_return_type_is_non_pod #foreign"));
        assert(has(BINDINGS, "MAX_THINGS :: 16;"));
        assert(has(BINDINGS, "operator+ :: (this: Geo.Vec2, o: Geo.Vec2) -> Geo.Vec2 #no_context {"));
        assert(has(BINDINGS, "// This class's own implementations of its virtual functions."));
        assert(has(BINDINGS, "shapes_format_CFormat :: (out: *u8, capacity: s32, fmt: *u8, __args: ..Any) -> s32 #foreign libshapes \"shapes_format\";"));
        assert(has(BINDINGS, "shapes_format :: (out: *u8, capacity: s32, fmt: string, __args: ..Any) -> s32 {"));

    // A printf-like function: the wrapper formats with Jai's print and passes the text as "%s".
    buffer: [32] u8;
    written := shapes_format(buffer.data, buffer.count, "%-%", 12, "ab");
    assert(written == 5);
    assert(to_string(buffer.data) == "12-ab");

    // Non-virtual methods, constructors, destructors.
    c: Geo.Counter;
    Geo.Counter.Constructor(*c, 5);
    Geo.Counter.add(*c, 7);
    print("counter = % (live %)\n", Geo.Counter.get(*c), live());
    assert(Geo.Counter.get(*c) == 12);
    assert(live() == 1);
    Geo.Counter.Destructor(*c);
    assert(live() == 0);

    v: Geo.Vec2;
    v.x = 3; v.y = 4;
    assert(Geo.Vec2.length_sq(*v) == 25);
    Geo.Vec2.scale(*v, 2);
    assert(v.x == 6 && v.y == 8);

    // Virtual calls through the vtable.
    s := new_circle(7, 2);
    print("name = %, area = %\n", to_string(Geo.Shape_name(s)), Geo.Shape_area(s));
    assert(Geo.Shape_area(s) == 12);
    assert(Geo.Shape.get_id(s) == 7);
    Geo.Shape_set_id(s, 9);
    assert(s.id == 9);

    circle := cast(*Geo.Circle) s;
    assert(Geo.Circle_circumference(circle) == 12);
    assert(Geo.vtable(s).area != null);
    delete_shape(s);

    d: Geo.Derived1;
    d.b1 = 3; d.d1 = 4;
    assert(Geo.Derived1.sum(*d) == 7);
    assert(d.b1 == 3);
    // By-value structs.
    a := Geo.Vec2.make(2);
    assert(a.x == 2 && a.y == 2);
    b: Geo.Vec2; b.x = 1; b.y = 5;
    sum := Geo.add_vec(a, b);
    assert(sum.x == 3 && sum.y == 7);
    sum2 := a + b;
    assert(sum2.x == 3 && sum2.y == 7);
    assert(a + b == sum);
    r := a - b;
    assert(r.x == 1 && r.y == -3);
    p := Geo.operator[](*a, 1);
    assert(p.* == 2);
    big := Geo.make_big(10);
    assert(big.v[0] == 10 && big.v[7] == 17);
    assert(Geo.sum_big(big) == 108);
    m := Geo.make_mixed(3, 0.5);
    assert(m.a == 3 && m.b == 0.5);
    assert(Geo.mixed_sum(m) == 3.5);
    before := live();
    cn := Geo.make_counter(41);
    assert(cn.value == 41);
    assert(live() == before + 1);
    Geo.Counter.Destructor(*cn);
    assert(live() == before);
    print("ok\n");

    } else {
        print("ok\n");
    }
}
