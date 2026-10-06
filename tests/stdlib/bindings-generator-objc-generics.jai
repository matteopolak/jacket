// Bindings_Generator on Objective-C lightweight generics: `@interface Box<T : id>` becomes
// `Box :: struct(T: Type)`, uses like `NSArray<Item *> *` become `*Group(*Item)`, a bare generic
// class takes `id`, and `Sub : Box<Item *>` inherits `Box(*Item)`. The classes are built with
// clang into a dylib at compile time and driven through the generated message sends.
// macOS only; prints ok (without checks) elsewhere or when clang/libclang is missing.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#import <objc/NSObject.h>

int generics_ping(void);

@interface Item : NSObject
@property (nonatomic) int tag;
@end

@interface Group<ObjectType> : NSObject
- (ObjectType)first;
- (void)add:(ObjectType)object;
@end

@interface Box<T : id> : NSObject
+ (instancetype)box;
- (instancetype)initWithItem:(T)item;
- (T)item;
- (void)setItem:(T)item;
- (Group<T> *)group;
- (Group *)plainGroup;
- (Group<Item *> *)itemGroup;
@end

@interface Sub : Box<Item *>
- (int)itemTag;
@end
END;

SOURCE :: #string END
#include "generics.h"
int generics_ping(void) { return 7; }

@implementation Item
@end

@implementation Group {
    id _first;
}
- (id)first { return _first; }
- (void)add:(id)object { if (!_first) _first = object; }
@end

@implementation Box {
    id _item;
}
+ (instancetype)box { return [[self alloc] init]; }
- (instancetype)initWithItem:(id)item { self = [super init]; if (self) _item = item; return self; }
- (id)item { return _item; }
- (void)setItem:(id)item { _item = item; }
- (Group *)group { Group *g = [[Group alloc] init]; [g add:_item]; return g; }
- (Group *)plainGroup { return [self group]; }
- (Group<Item *> *)itemGroup { return [self group]; }
@end

@implementation Sub
- (int)itemTag { return ((Item *)[self item]).tag; }
@end
END;

build_and_generate :: () -> string {
    #if OS == .MACOS {
        dir := "/tmp/jai_bindgen_objc_generics";
        make_directory_if_it_does_not_exist(dir);
        header_path := tprint("%/generics.h", dir);
        source_path := tprint("%/generics.m", dir);
        library_path := tprint("%/libgenerics.dylib", dir);
        if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

        result, output := run_command("clang", "-shared", "-fPIC", "-fno-objc-arc", "-o", library_path, source_path, "-lobjc", "-Wno-deprecated-declarations", capture_and_return_output = true);
        if result.exit_code != 0 return "";

        opts: Generate_Bindings_Options;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-x", "objective-c");
        array_add(*opts.libpaths, dir);
        array_add(*opts.libnames, "libgenerics");
        builder, ok := generate_bindings(opts);
        if !ok return "";
        return builder_to_string(*builder);
    } else {
        return "";
    }
}

BINDINGS :: #run build_and_generate();
HAVE_OBJC :: BINDINGS.count > 0;

#if HAVE_OBJC {
    #import "Objective_C";
    #insert BINDINGS;
}

has :: (text: string, needle: string) -> bool {
    return contains(text, needle);
}

main :: () {
    #if HAVE_OBJC {
        // The generated shape.
        assert(has(BINDINGS, "Group :: struct(ObjectType: Type) {"));
        assert(has(BINDINGS, "Box :: struct(T: Type) {"));
        assert(has(BINDINGS, "item :: inline (self: *Box(T)) -> T {"));
        assert(has(BINDINGS, "group :: inline (self: *Box(T)) -> *Group(T) {"));
        assert(has(BINDINGS, "plainGroup :: inline (self: *Box(T)) -> *Group(id) {"));
        assert(has(BINDINGS, "itemGroup :: inline (self: *Box(T)) -> *Group(*Item) {"));
        assert(has(BINDINGS, "box :: inline (class: Class) -> *Box(T) {"));
        assert(has(BINDINGS, "initWithItem :: inline (self: *$instancetype/Box(T), item: T) -> *instancetype {"));
        assert(has(BINDINGS, "Sub :: struct {"));
        assert(has(BINDINGS, "#as using box: Box(*Item);"));

        assert(generics_ping() == 7); // Loads the library, and with it the classes.

        item := objc_alloc(class(Item));
        Item.setTag(cast(*Item) item, 42);

        b := Box(*Item).box(class(Box(*Item)));
        assert(Box(*Item).item(b) == null);
        Box(*Item).setItem(b, cast(*Item) item);
        assert(type_of(Box(*Item).item(b)) == *Item);
        assert(Item.tag(Box(*Item).item(b)) == 42);

        // Generic results keep their arguments.
        g := Box(*Item).itemGroup(b);
        assert(type_of(g) == *Group(*Item));
        assert(Group(*Item).first(g) == cast(*Item) item);
        assert(Group(id).first(Box(*Item).plainGroup(b)) == item);

        // A subclass of a specialised generic class inherits `Box(*Item)`.
        s := cast(*Sub) Box(*Item).initWithItem(objc_alloc(class(Sub)), cast(*Item) item);
        assert(Sub.itemTag(s) == 42);
        assert(Box(*Item).item(s) == cast(*Item) item);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
