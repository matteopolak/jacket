// SHA-2 and MD5 from smari/jai-crypto over a generated buffer. Copied into the project as its entry.
#import "Basic";
#load "module.jai";

main :: () {
    data := alloc_string(4_000_000);
    state: u32 = 2463534242;

    for 0..data.count - 1 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        data[it] = cast,trunc(u8) state;
    }

    for round: 0..2 {
        piece := advance(data, round * 1000);
        print("% % %\n", sha256(piece), sha512(piece), md5(piece));
    }
}
