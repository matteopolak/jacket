#import "Basic";

main :: () {
    low: u8 = 64;
    high: u8 = 31;
    combined := cast(u32) low | (cast(u32) high << 8);

    assert(combined == 8000);
    print("ok\n");
}
