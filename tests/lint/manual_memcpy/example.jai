#import "Basic";

copy_all :: (dst: *int, src: [] int, count: int) {
    for i: 0..count - 1 {
        dst[i] = src[i];
    }
}

main :: () {
    values := int.[1, 2, 3, 4];
    out: [4] int;

    copy_all(out.data, values, 4);
    print("%\n", out);
}
