#import "Basic";
Apple :: #import "Bindings_Generator"(TARGET_OS = .IOS);

main :: () {
    name: string = "__original";
    Apple.fix_macos_foreign_name(*name);
    assert(name == "_original");
    print("ok\n");
}
