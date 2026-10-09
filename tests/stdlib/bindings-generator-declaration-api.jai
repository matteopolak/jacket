// Bindings_Generator: enum values are `*Declaration`s. A visitor sees each one after its enum,
// can leave one out, and can point a `Literal` of kind `.ENUM` at one (as Vulkan generators
// do for `sType` defaults). A library's Jai name is `Library_Info.identifier`, which
// `will_print_bindings` may change (sgpu's Vulkan generator).
#import "Basic";
#import "String";
#import "File";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string C
typedef enum Kind { KIND_NONE, KIND_LINE = 3, KIND_CIRCLE, KIND_INTERNAL_COUNT } Kind;
typedef enum Signed_Kind { SIGNED_KIND_LOW = -2, SIGNED_KIND_HIGH = 7 } Signed_Kind;
typedef struct Shape { Kind kind; float size; } Shape;
#define MODE_FAST 1
#define MODE_SAFE (MODE_FAST << 1)
int shape_area(Shape *shape);
C;

kind_enum: *Enum;
values_seen: int;
parents_ok := true;

visitor :: (decl: *Declaration, parent: *Declaration) -> Declaration_Visit_Result {
    if decl.kind == .ENUM && decl.name == "Kind" kind_enum = cast(*Enum) decl;

    if parent && parent.kind == .ENUM && parent.name == "Kind" {
        values_seen += 1;
        if decl.parent != parent parents_ok = false;
        if decl.name == "KIND_INTERNAL_COUNT" decl.decl_flags |= .OMIT_FROM_OUTPUT;
    }

    // `Shape.kind` defaults to `.LINE`.
    if parent && parent.kind == .STRUCT && parent.name == "Shape" && decl.name == "kind" {
        value: *Declaration;
        for kind_enum.enumerates {
            if it.output_name == "LINE" {
                value = it;
                break;
            }
        }
        assert(value != null);
        literal := New(Literal);
        literal.literal_kind = .ENUM;
        literal.enum_type = kind_enum;
        literal.enum_value = value;
        decl.expression = literal;
    }
    return .RECURSE;
}

// Runs at compile time: libclang is the compiler's, and executables cannot call it.
check_bindings :: () {
    dir := scratch_directory("bindgen-declaration-api");
    hide_scratch_directory_in_log(dir);
    defer delete_directory(dir);
    header_path := tprint("%/test.h", dir);
    output_path := tprint("%/test.jai", dir);
    assert(write_entire_file(header_path, HEADER));

    opts: Generate_Bindings_Options;
    array_add(*opts.source_files, header_path);
    array_add(*opts.generate_enums_from_macros_with_prefixes, "MODE_");
    opts.generate_library_declarations = false;
    opts.visitor = visitor;
    array_add(*opts.libraries, .{ filename = "libshapes_missing", identifier = "shapes" });
    opts.will_print_bindings = () {
        lib := context.generator.libraries[0];
        assert(lib.identifier == "shapes" && lib.name == "libshapes_missing");
        lib.identifier = "renamed_shapes";
    };
    assert(generate_bindings(opts, output_path));

    assert(kind_enum != null);
    assert(kind_enum.enumerates.count == 4);
    assert(values_seen == 4);
    assert(parents_ok);
    first := kind_enum.enumerates[0];
    assert(first.name == "KIND_NONE" && first.output_name == "NONE");
    assert(first.expression != null && first.expression.kind == .LITERAL);

    text, ok := read_entire_file(output_path);
    assert(ok);
    assert(contains(text, "LINE   :: 3;"));
    assert(contains(text, "CIRCLE :: 4;"));
    assert(!contains(text, "INTERNAL_COUNT"));
    assert(contains(text, "LOW  :: -2;"));
    assert(contains(text, "kind: Kind = .LINE;"));
    assert(contains(text, "SAFE :: FAST << 1;"));
    assert(contains(text, "#foreign renamed_shapes;"));
}

#run check_bindings();

main :: () {
    print("ok\n");
}
