#import "Basic";

copy_all :: (dst: *int, src: [] int, count: int) {
    memcpy(dst, src.data, count * size_of(int));
}

main :: () {
    values := int.[1, 2, 3, 4];
    out: [4] int;

    copy_all(out.data, values, 4);
    print("%\n", out);
}
