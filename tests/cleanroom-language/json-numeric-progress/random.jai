#import "Basic";
#import "Extensions/Json";

main :: () {
    state: u64 = 0xd1b54a32d192ed03;

    print("ready\n");

    for index: 0..127 {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        bits := (state & 0x800fffffffffffff) | (((state >> 52) % 2047) << 52);
        value := cast,force(float64) bits;
        encoded := to_json(value);
        decoded, error := from_json(float64, encoded);

        assert(error.kind == .NONE);
        assert(cast,force(u64) decoded == bits);
        if index % 16 == 15 print("roundtrips %\n", index + 1);
    }

    print("done\n");
}
