#import "Basic";

Pixel :: struct {
    r: u8;
    g: u8;
    b: u8;
    a: u8;
}

render :: () -> int {
    scratch: [1 << 20] u8;

    pixels: [100 * 100] Pixel;

    total := 0;

    for scratch total += it;
    for pixels total += it.r;

    return total;
}

main :: () {
    print("%\n", render());
}
