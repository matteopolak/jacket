// copy handles overlapping ranges in both directions for every pair of offsets from 0 to 15 and counts from 0 to 20.
#import "Basic";
Memory :: #import "Memory";

main :: () {
    for dst_off: 0..15 {
        for src_off: 0..15 {
            for count: 0..20 {
                buf: [64] u8;
                for *b, i: buf b.* = cast(u8) (i + 100);

                // Expected result, built by copying through a temporary array.
                reference := buf;
                tmp: [64] u8;
                i := 0;

                while i < count {
                    tmp[i] = reference[src_off + i];
                    i += 1;
                }

                i = 0;
                while i < count {
                    reference[dst_off + i] = tmp[i];
                    i += 1;
                }

                Memory.copy(buf.data + dst_off, buf.data + src_off, count);

                for k: 0..63 assert(buf[k] == reference[k]);
            }
        }
    }

    print("ok\n");
}
