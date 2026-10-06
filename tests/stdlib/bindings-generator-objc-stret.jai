// Bindings_Generator on Objective-C methods that return structs: for `cpu = .X64` a struct the
// SysV ABI returns in memory (over 16 bytes) is sent with `objc_msgSend_stret`; small structs and
// everything else use `objc_msgSend`. arm64 never uses the `_stret` entry points. The x64 bindings
// are also inserted here to check that they type check (the Objective_C module aliases
// `objc_msgSend_stret` on arm64 hosts); they are not called. macOS only; skips (printing ok)
// elsewhere or when libclang is missing.
#import "Basic";
#import "String";
#import "File";
#import "Process"; // jailint: allow(unused_import) only the macOS bindings use it
#import "Bindings_Generator";

HEADER :: #string END
struct Big { long a, b, c; };
struct Edge { long a, b; };
struct Small { int a, b; };
typedef struct Big BigAlias;

__attribute__((objc_root_class))
@interface Maker
- (struct Big)makeBig;
- (BigAlias)makeBigAlias;
- (struct Edge)makeEdge;
- (struct Small)makeSmall;
- (long)plain;
- (double)ratio;
+ (struct Big)classBig:(int)x;
@end
END;

generate :: (cpu: type_of(Generate_Bindings_Options.cpu)) -> string {
    #if OS == .MACOS {
        dir := "/tmp/jai_bindgen_objc_stret";
        make_directory_if_it_does_not_exist(dir);
        header_path := tprint("%/maker.h", dir);
        if !write_entire_file(header_path, HEADER) return "";

        opts: Generate_Bindings_Options;
        opts.cpu = cpu;
        array_add(*opts.source_files, header_path);
        array_add(*opts.extra_clang_arguments, "-x", "objective-c");
        builder, ok := generate_bindings(opts);
        if !ok return "";
        return builder_to_string(*builder);
    } else {
        return "";
    }
}

X64 :: #run generate(.X64);
ARM64 :: #run generate(.ARM64);
HAVE_OBJC :: X64.count > 0 && ARM64.count > 0;

#if HAVE_OBJC && OS == .MACOS {
    #insert X64;
}

count_of :: (text: string, needle: string) -> s64 {
    n := 0;
    rest := text;
    while true {
        i := find_index_from_left(rest, needle);
        if i < 0 break;
        n += 1;
        rest = slice(rest, i + needle.count, rest.count - i - needle.count);
    }
    return n;
}

main :: () {
    #if HAVE_OBJC {
        // x64: Big (3 longs), BigAlias and the class method need _stret; Edge (16 bytes) is
        // returned in registers.
        assert(count_of(X64, "_func = xx objc_msgSend_stret;") == 3);
        assert(count_of(X64, "_func = xx objc_msgSend;") == 4);
        assert(!contains(ARM64, "objc_msgSend_stret"));
        assert(count_of(ARM64, "_func = xx objc_msgSend;") == 7);
        print("ok\n");
    } else {
        print("ok\n");
    }
}
