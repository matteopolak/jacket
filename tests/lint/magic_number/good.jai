#import "Basic";

LIMIT :: 37;
SCALE :: 1.0472 * 2;

Settings :: struct {
    retries := 5;
    ratio := 0.75;
}

Color :: enum {
    RED :: 7;
    BLUE :: 11;
}

lookup :: (values: [] int, n: int = 37) -> int {
    buffer: [48] u8;
    table := int.[3, 5, 7, 11];
    mask := n & 0x3f;
    shifted := (n << 3) | (n >> 5);
    hex := n + 0xFF;
    total := table[3] + values.count;

    if total > LIMIT total -= 1;
    if total < -1 || total == 2 total = 0;

    total = total * 4 + 1024 - 64;
    total = total * 10 + 100 / 1000;
    half := cast(float) total * 0.5 + 0.25 * 2.0;
    seed := 12345;
    case_value := 0;

    if total == {
        case 42; case_value = 1;
    }

    computed := #run 37 * 41;
    STEP :: 3 * 13;
    register := n;

    #if CPU == .X64 {
        #asm { add register, 37; }
    }

    print("%\n", 13, 99);

    return mask + shifted + hex + total + cast(int) half + seed + case_value + computed + STEP + buffer.count + register;
}

main :: () {
    s: Settings;
    print("%\n", lookup(.[1, 2, 3]) + s.retries);
}
