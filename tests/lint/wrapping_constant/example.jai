#import "Basic";

bucket :: (id: s32) -> s32 {
    return id % 0x8000_0000;
}

main :: () {
    print("%\n", bucket(5));
}
