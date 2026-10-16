#import "Basic";

original :: #library "./liboriginal_resolved";
original_resolved :: () -> s32 #foreign original;

main :: () {
    assert(original_resolved() == 37);
    print("ok\n");
}
