#import "Basic";

copy_values :: (dest: [] s32, values: [] s32) {
    memcpy(dest.data, values.data, values.count);
}

main :: () {
    a: [4] s32;
    b := s32.[1, 2, 3, 4];
    copy_values(a, b);
    print("%\n", a);
}
