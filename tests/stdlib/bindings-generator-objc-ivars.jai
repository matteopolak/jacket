// Bindings_Generator on Objective-C instance variables: the ivars declared in an
// `@interface { ... }` block become data members of the object struct, after the superclass
// member, so Jai code can read and write them at the offsets clang laid out. The classes are
// built with clang into a dylib at compile time; Objective-C reads what Jai writes and vice
// versa. macOS only; prints ok (without checks) elsewhere or when clang/libclang is missing.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";
#import "Scratch_Directory";

HEADER :: #string OBJC
#import <objc/NSObject.h>

int ivars_ping(void);

@interface Point : NSObject {
@public
    int _x;
    char _flag;
    double _ratio;
    id _owner;
    int *_buffer;
    void (*_callback)(int);
@private
    long _hidden;
}
+ (instancetype)point;
- (int)x;
- (char)flag;
- (double)ratio;
- (long)hidden;
- (void)setHidden:(long)value;
@end

@interface Point3 : Point {
@public
    float _z;
}
- (float)z;
@end
OBJC;

SOURCE :: #string OBJC
#include "ivars.h"
int ivars_ping(void) { return 3; }

@implementation Point
+ (instancetype)point { return [[self alloc] init]; }
- (int)x { return _x; }
- (char)flag { return _flag; }
- (double)ratio { return _ratio; }
- (long)hidden { return _hidden; }
- (void)setHidden:(long)value { _hidden = value; }
@end

@implementation Point3
- (float)z { return _z; }
@end
OBJC;

build_and_generate :: () -> string {
    #if OS == .MACOS {
        // Kept: the program loads the library from here when it runs.
        dir := scratch_directory("bindgen-objc-ivars");
        header_path := tprint("%/ivars.h", dir);
        source_path := tprint("%/ivars.m", dir);
        library_path := tprint("%/libivars.dylib", dir);
        if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

        result := run_command("clang", "-shared", "-fPIC", "-fno-objc-arc", "-o", library_path, source_path, "-lobjc", "-Wno-deprecated-declarations", capture_and_return_output = true);
        if result.exit_code != 0 return "";

        opts: Generate_Bindings_Options;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-x", "objective-c");
        array_add(*opts.libpaths, dir);
        array_add(*opts.libnames, "libivars");
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
        // The generated shape: ivars follow the superclass member, in declaration order.
        assert(has(BINDINGS, "Point :: struct {"));
        assert(has(BINDINGS, "_x:        s32;"));
        assert(has(BINDINGS, "_ratio:    float64;"));
        assert(has(BINDINGS, "_owner:    id;"));
        assert(has(BINDINGS, "_buffer:   *s32;"));
        assert(has(BINDINGS, "_hidden:   s64;"));
        assert(has(BINDINGS, "#as using point: Point;"));
        assert(has(BINDINGS, "_z: float;"));

        assert(ivars_ping() == 3); // Loads the library, and with it the classes.

        p := Point.point(class(Point));
        p._x = 11;
        p._flag = 7;
        p._ratio = 2.5;
        assert(Point.x(p) == 11);
        assert(Point.flag(p) == 7);
        assert(Point.ratio(p) == 2.5);

        // Private ivars are listed too; both sides see the same bytes.
        Point.setHidden(p, 123456789);
        assert(p._hidden == 123456789);
        p._hidden = -5;
        assert(Point.hidden(p) == -5);

        p._owner = xx p;
        assert(p._owner == xx p);
        assert(p._buffer == null);
        assert(p._callback == null);

        // A subclass's ivars follow the superclass ones.
        q := cast(*Point3) Point.point(class(Point3));
        q._z = 1.5;
        q._x = 4;
        assert(Point3.z(q) == 1.5);
        assert(Point.x(q) == 4);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
