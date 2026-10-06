// Bindings_Generator on an Objective-C header: classes with inheritance, a protocol,
// a category, properties, class methods, `instancetype` returns, floating point and `BOOL`
// results. The classes are built with clang into a dylib at compile time and called through
// the generated message sends (`objc_msgSend` with the method's signature), both interpreted
// and natively. macOS only; skips (printing ok) elsewhere or when clang/libclang is missing.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#import <objc/NSObject.h>

int counter_ping(void);

@protocol Greeter
- (int)greetCount;
@end

@interface Counter : NSObject <Greeter>
@property (nonatomic) int value;
+ (instancetype)counterWithValue:(int)v;
- (instancetype)initWithValue:(int)v;
- (int)add:(int)n to:(int)m;
- (double)scale:(double)f;
- (BOOL)isBig;
- (id)selfAsId;
- (void)reset;
@end

@interface Counter (Extras)
- (int)doubled;
@end

@interface Sub : Counter
- (int)subValue;
@end
END;

SOURCE :: #string END
#include "counter.h"
int counter_ping(void) { return 42; }

@implementation Counter
+ (instancetype)counterWithValue:(int)v { return [[self alloc] initWithValue:v]; }
- (instancetype)initWithValue:(int)v { self = [super init]; if (self) _value = v; return self; }
- (int)add:(int)n to:(int)m { return n + m + _value; }
- (double)scale:(double)f { return _value * f; }
- (BOOL)isBig { return _value > 100; }
- (id)selfAsId { return self; }
- (void)reset { _value = 0; }
- (int)greetCount { return _value * 10; }
@end

@implementation Counter (Extras)
- (int)doubled { return _value * 2; }
@end

@implementation Sub
- (int)subValue { return self.value + 1000; }
@end
END;

build_and_generate :: () -> string {
    #if OS == .MACOS {
        dir := "/tmp/jai_bindgen_objc";
        make_directory_if_it_does_not_exist(dir);
        header_path := tprint("%/counter.h", dir);
        source_path := tprint("%/counter.m", dir);
        library_path := tprint("%/libcounter.dylib", dir);
        if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

        result, output := run_command("clang", "-shared", "-fPIC", "-fno-objc-arc", "-o", library_path, source_path, "-lobjc", "-Wno-deprecated-declarations", capture_and_return_output = true);
        if result.exit_code != 0 return "";

        opts: Generate_Bindings_Options;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-x", "objective-c");
        array_add(*opts.libpaths, dir);
        array_add(*opts.libnames, "libcounter");
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
        assert(has(BINDINGS, "Counter :: struct {"));
        assert(has(BINDINGS, "#as using nsobject: NSObject;"));
        assert(has(BINDINGS, "#as using greeter: Greeter;"));
        assert(has(BINDINGS, "counterWithValue :: inline (class: Class, v: s32) -> *Counter {"));
        assert(has(BINDINGS, "initWithValue :: inline (self: *$instancetype/Counter, v: s32) -> *instancetype {"));
        assert(has(BINDINGS, "_func = xx objc_msgSend;"));
        assert(has(BINDINGS, "doubled :: inline (self: *Counter) -> s32 {"));
        assert(has(BINDINGS, "Sub :: struct {"));
        assert(has(BINDINGS, "#as using counter: Counter;"));

        assert(counter_ping() == 42); // Loads the library, and with it the classes.

        c := Counter.counterWithValue(class(Counter), 5);
        assert(c != null);
        assert(Counter.value(c) == 5);
        Counter.setValue(c, 7);
        assert(Counter.value(c) == 7);
        assert(Counter.add(c, 1, 2) == 10);
        assert(Counter.scale(c, 0.5) == 3.5);
        assert(!truth(Counter.isBig(c)));
        Counter.setValue(c, 500);
        assert(truth(Counter.isBig(c)));
        assert(Counter.doubled(c) == 1000); // From the category.
        assert(Greeter.greetCount(c) == 5000); // Through the protocol.
        assert(Counter.selfAsId(c) == xx c);
        Counter.reset(c);
        assert(Counter.value(c) == 0);

        // `instancetype` follows the receiver: an `init` of a subclass returns the subclass.
        s := Counter.initWithValue(objc_alloc(Sub), 3);
        assert(type_of(s) == *Sub);
        assert(Sub.subValue(s) == 1003);
        assert(Counter.value(s) == 3); // Inherited through `#as using`.
        print("ok\n");
    } else {
        print("ok\n");
    }
}

// Objective-C `BOOL` is `bool` on arm64 and `signed char` (s8) on x86-64; the bindings follow clang.
truth :: (b: $T) -> bool {
    #if T == bool return b;
    else return b != 0;
}
