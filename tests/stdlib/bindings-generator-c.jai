// Bindings_Generator on a small C header written at compile time.
#import "Basic";
#import "String";
#import "File";
#import "Bindings_Generator";

HEADER :: #string END
#define LIMIT 5
#define MASK (LIMIT << 2)
#define NAME "hello"
#define UNKNOWN_CALL(x) ((x) + 1)

typedef struct Node { int value; struct Node *next; float weights[4]; void (*callback)(int code, struct Node *node); } Node;
typedef enum Color { COLOR_RED, COLOR_GREEN = 5, COLOR_BLUE } Color;
typedef unsigned int Flags;
union Number { int i; float f; };
struct Packed { unsigned int a : 3; unsigned int b : 5; int tail; };
typedef struct Opaque_T *Opaque;
int add(int a, int b);
void log_message(const char *fmt, ...);
Opaque make_opaque(Node *node, Color color, Flags flags);
END;

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

main :: () {
    dir := "/tmp";
    header_path := tprint("%/jai_bindgen_c_test.h", dir);
    output_path := tprint("%/jai_bindgen_c_test.jai", dir);
    assert(write_entire_file(header_path, HEADER));

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    opts.generate_library_declarations = false;
    array_add(*opts.libraries, .{filename = "does_not_exist", identifier = "lib"});
    assert(generate_bindings(opts, output_path));

    text, ok := read_entire_file(output_path);
    assert(ok);

    assert(has(text, "LIMIT :: 5;"));
    assert(has(text, "MASK :: LIMIT << 2;"));
    assert(has(text, "NAME :: \"hello\";"));
    assert(!has(text, "UNKNOWN_CALL"));
    assert(has(text, "Node :: struct {"));
    assert(has(text, "next:     *Node;"));
    assert(has(text, "weights:  [4] float;"));
    assert(has(text, "callback: #type (code: s32, node: *Node) -> void #c_call;"));
    assert(has(text, "Color :: enum u32 {"));
    assert(has(text, "GREEN :: 5;"));
    assert(has(text, "COLOR_GREEN :: GREEN;"));
    assert(has(text, "Flags :: u32;"));
    assert(has(text, "Number :: union {"));
    assert(has(text, "__bitfield: u8;"));
    assert(has(text, "Packed_get_a :: (s: *Packed) -> u32 {"));
    assert(has(text, "Opaque_T :: struct {"));
    assert(has(text, "Opaque :: *Opaque_T;"));
    assert(has(text, "add :: (a: s32, b: s32) -> s32 #foreign lib;"));
    assert(has(text, "log_message_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign lib;"));
    assert(has(text, "log_message :: (fmt: string, __args: ..Any)"));
    assert(has(text, "make_opaque :: (node: *Node, color: Color, flags: Flags) -> Opaque #foreign lib;"));
    assert(has(text, "assert(size_of(Node) == 40"));
    print("ok\n");
}
