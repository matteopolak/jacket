#import "Basic";

bucket :: (id: s32) -> s32 {
    return id % 0x7fff_ffff;
}

main :: () {
    print("%\n", bucket(5));
}
