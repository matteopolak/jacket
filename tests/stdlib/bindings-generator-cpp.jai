// Bindings_Generator on a small C++ header: flattened namespaces, mangled
// names, reference defaults wrapped as values, visitor and printing hooks.
#import "Basic";
#import "String";
#import "File";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string END
struct Vec2 { float x, y; };
namespace Gui {
    bool Button(const char* label, const Vec2& size);
    bool Begin(const char* name, bool* p_open = nullptr, int flags = 0);
    float Scale(float s = 1.5f, int n = 3);
}

// Default arguments that are temporaries, null casts and bools; macros that name types.
struct P2 { float x, y; P2() : x(0), y(0) {} P2(float a, float b) : x(a), y(b) {} };
struct WC;
typedef unsigned int Id;
enum E_ { E_None = 0, E_A = 1 };
void f1(Id id, const P2& size = P2(0, 0), const WC* wc = nullptr);
void f2(const WC* wc = (const WC*)0, bool c = true, E_ e = E_None);
void f3(P2 a = P2(1.0f, -1.0f), const P2& b = P2());
#define GUI_TAB (4)
#define GUI_NEG (-1.0)
#define GUI_ID_ANY ((Id)0)
#define GUI_ID_NONE ((Id)-1)
#define GUI_INDEX_TYPE int
#define GUI_ID_TYPE Id
END;

visited: int;

visitor :: (decl: *Declaration, parent: *Declaration) -> Declaration_Visit_Result {
    if decl.kind == .FUNCTION && decl.name == "Scale" {
        decl.decl_flags |= .OMIT_FROM_OUTPUT;
        return .STOP;
    }
    visited += 1;
    return .RECURSE;
}

// Runs at compile time: libclang is the compiler's, and executables cannot call it.
check_bindings :: () {
    // The WASM target (the browser playground) loads no native libraries, so no libclang.
    #if OS == .WASM return;
    dir := scratch_directory("bindgen-cpp");
    defer delete_directory(dir);
    header_path := tprint("%/test.h", dir);
    output_path := tprint("%/test.jai", dir);
    assert(write_entire_file(header_path, HEADER));

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.extra_clang_arguments, "-x", "c++");
    array_add(*opts.flatten_namespaces, "Gui");
    opts.generate_library_declarations = false;
    opts.generate_compile_time_struct_checks = false;
    opts.visitor = visitor;
    assert(generate_bindings(opts, output_path));

    text, ok := read_entire_file(output_path);
    assert(ok);
    assert(visited > 0);
    assert(contains(text, "Button :: (label: *u8, size: *Vec2) -> bool #foreign \"_ZN3Gui6ButtonEPKcRK4Vec2\";"));
    assert(contains(text, "Begin :: (name: *u8, p_open: *bool = null, flags: s32 = 0) -> bool #foreign"));
    assert(!contains(text, "Scale"));
    assert(contains(text, "f1 :: (id: Id, size: P2 = P2.{0.0, 0.0}, wc: *WC = null) #no_context"));
    assert(contains(text, "f2 :: (wc: *WC = null, c: bool = true, e: E_ = .None)"));
    assert(contains(text, "f3 :: (a: P2 = P2.{1.0, -1.0}, b: P2 = P2.{}) #no_context"));
    assert(contains(text, "GUI_TAB :: 4;"));
    assert(contains(text, "GUI_NEG :: -1.0;"));
    assert(contains(text, "GUI_ID_ANY :: cast(Id) 0;"));
    assert(contains(text, "GUI_ID_NONE :: cast,trunc(Id) -1;"));
    assert(contains(text, "GUI_INDEX_TYPE :: s32;"));
    assert(contains(text, "GUI_ID_TYPE :: Id;"));
}

#run check_bindings();

main :: () {
    print("ok\n");
}
