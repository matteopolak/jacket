#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

copy_all :: (dst: *int, src: [] int, count: int) {
    for i: 0..count - 1 {
        dst[i] = src[i];
    }
}

copy_points :: (dst: [] Point, src: [] Point) {
    for 0..src.count - 1 dst[it] = src[it];
}

copy_fixed :: () {
    a: [8] u8;
    b: [8] u8;

    for j: 0..7 {
        a[j] = b[j];
    }
}

main :: () {
    values: [4] int;
    out: [4] int;

    copy_all(out.data, values, 4);
    copy_fixed();
    print("%\n", out);
}
