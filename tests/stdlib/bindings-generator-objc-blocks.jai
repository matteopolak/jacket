// Bindings_Generator on Objective-C block types: `void (^)(int)` parameters, results, properties
// and typedefs become pointers to a per-signature `Block_<result>_<args>` struct (the block
// header with `invoke` taking the block first), shared between uses of the same signature.
// The classes are built with clang into a dylib at compile time; Jai calls blocks that
// Objective-C returns and passes them back. macOS only; prints ok (without checks) elsewhere
// or when clang/libclang is missing.
#import "Basic";
#import "String";
#import "File";
#import "Process";
#import "Bindings_Generator";

HEADER :: #string END
#import <objc/NSObject.h>

int blocks_ping(void);

#pragma clang assume_nonnull begin
typedef int (^Mapper)(int value);
typedef void (^Handler)(int code, double weight);

@interface Worker : NSObject
@property (nonatomic, copy) Handler onDone;
@property (nonatomic, readonly) int lastCode;
- (Mapper)multiplier:(int)k;
- (int)apply:(Mapper)mapper to:(int)value;
- (int)applyInline:(int (^)(int))mapper to:(int)value;
- (void)fire:(Handler)handler code:(int)code;
- (void (^)(int, double))recorder;
- (void)setCompletion:(void (^ _Nullable)(void))completion;
- (double)weight;
@end
#pragma clang assume_nonnull end
END;

SOURCE :: #string END
#include <Block.h>
#include "blocks.h"
int blocks_ping(void) { return 5; }

@implementation Worker {
    int _lastCode;
    double _weight;
    Handler _onDone;
}
- (Handler)onDone { return _onDone; }
- (void)setOnDone:(Handler)h { if (_onDone) Block_release(_onDone); _onDone = Block_copy(h); }
- (int)lastCode { return _lastCode; }
- (double)weight { return _weight; }
- (Mapper)multiplier:(int)k { return Block_copy(^int(int v) { return v * k; }); }
- (int)apply:(Mapper)mapper to:(int)value { return mapper(value); }
- (int)applyInline:(int (^)(int))mapper to:(int)value { return mapper(value) + 1; }
- (void)fire:(Handler)handler code:(int)code { handler(code, 0.5); }
- (void (^)(int, double))recorder { return Block_copy(^(int c, double w) { _lastCode = c; _weight = w; }); }
- (void)setCompletion:(void (^)(void))completion { if (completion) completion(); }
@end
END;

build_and_generate :: () -> string {
    #if OS == .MACOS {
        dir := "/tmp/jai_bindgen_objc_blocks";
        make_directory_if_it_does_not_exist(dir);
        header_path := tprint("%/blocks.h", dir);
        source_path := tprint("%/blocks.m", dir);
        library_path := tprint("%/libblocks.dylib", dir);
        if !write_entire_file(header_path, HEADER) || !write_entire_file(source_path, SOURCE) return "";

        result, output := run_command("clang", "-shared", "-fPIC", "-fno-objc-arc", "-o", library_path, source_path, "-lobjc", "-Wno-deprecated-declarations", capture_and_return_output = true);
        if result.exit_code != 0 return "";

        opts: Generate_Bindings_Options;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-x", "objective-c");
        array_add(*opts.libpaths, dir);
        array_add(*opts.libnames, "libblocks");
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

has :: (text: string, needle: string) -> bool { return contains(text, needle); }

count_of :: (text: string, needle: string) -> s64 {
    count := 0;
    rest := text;
    while true {
        index := find_index_from_left(rest, needle);
        if index < 0 break;
        count += 1;
        rest = slice(rest, index + needle.count, rest.count - index - needle.count);
    }
    return count;
}

main :: () {
    #if HAVE_OBJC {
        // The generated shape: one struct per signature, shared by all its uses.
        assert(has(BINDINGS, "Block_s32_s32 :: struct {"));
        assert(has(BINDINGS, "invoke:     #type (block: *Block_s32_s32, arg0: s32) -> s32 #c_call;"));
        assert(has(BINDINGS, "Mapper :: *Block_s32_s32;"));
        assert(has(BINDINGS, "Handler :: *Block_void_s32_float64;"));
        assert(has(BINDINGS, "invoke:     #type (block: *Block_void_s32_float64, arg0: s32, arg1: float64) -> void #c_call;"));
        assert(has(BINDINGS, "applyInline :: inline (self: *Worker, mapper: *Block_s32_s32, value: s32) -> s32 {"));
        assert(has(BINDINGS, "recorder :: inline (self: *Worker) -> *Block_void_s32_float64 {"));
        assert(has(BINDINGS, "setCompletion :: inline (self: *Worker, completion: *Block_void) {"));
        assert(count_of(BINDINGS, "Block_s32_s32 :: struct {") == 1);
        assert(count_of(BINDINGS, "Block_void_s32_float64 :: struct {") == 1);

        assert(blocks_ping() == 5);   // Loads the library, and with it the classes.

        w := objc_alloc(class(Worker));
        w = objc_init(w);

        // Blocks returned by Objective-C are called through `invoke`.
        triple := Worker.multiplier(cast(*Worker) w, 3);
        assert(triple != null);
        assert(triple.invoke(triple, 14) == 42);

        // ... and can be passed back, as a typedef'd or an inline block parameter.
        assert(Worker.apply(cast(*Worker) w, triple, 5) == 15);
        assert(Worker.applyInline(cast(*Worker) w, triple, 5) == 16);

        // A void block with several parameters stores into the object.
        rec := Worker.recorder(cast(*Worker) w);
        rec.invoke(rec, 77, 1.25);
        assert(Worker.lastCode(cast(*Worker) w) == 77);
        assert(Worker.weight(cast(*Worker) w) == 1.25);
        Worker.fire(cast(*Worker) w, rec, 9);
        assert(Worker.lastCode(cast(*Worker) w) == 9);
        assert(Worker.weight(cast(*Worker) w) == 0.5);

        // Properties: a block-typed property round trips through the setter and getter.
        Worker.setOnDone(cast(*Worker) w, rec);
        done := Worker.onDone(cast(*Worker) w);
        assert(done != null);
        done.invoke(done, 31, 2.0);
        assert(Worker.lastCode(cast(*Worker) w) == 31);

        Worker.setCompletion(cast(*Worker) w, null);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
