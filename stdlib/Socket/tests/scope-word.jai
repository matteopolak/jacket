// Exact production native-word arithmetic; no Windows backend is emulated.
#import "Basic";
#load "../../internal/net/scope-word.jai";

main :: () {
    for high: 0..15 {
        for value: u32.[0, 1, 0x01234567, 0x0fffffff, 0x10000000, 0xffffffff] {
            word := cast(u32) high << 28;
            changed := with_zone(word, value);

            assert(level(changed) == high && zone(changed) == (value & 0x0fffffff));

            for new_level: 0..255 {
                updated := with_level(changed, cast(u32) new_level);
                assert(zone(updated) == zone(changed) && level(updated) == (new_level & 15));
            }
        }
    }

    assert(with_zone(0xa1234567, 0xffffffff) == 0xafffffff);
    assert(with_level(0xafffffff, 0x12345678) == 0x8fffffff);
    print("ok\n");
}
