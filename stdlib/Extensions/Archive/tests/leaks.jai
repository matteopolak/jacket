#import "Basic";
#import "Extensions/Archive";
#import "Extensions/Testing";
#load "../../test-support/counting-allocator.jai";

ownership :: (t: *Test) {
    for format: Archive_Format.[.ZIP, .TAR, .TAR_GZIP] {
        tally: Tally;

        {
            push_allocator(tally_proc, *tally);
            items := Archive_Item.[.{ name = "name", data = cast([] u8) "payload payload", method = .DEFLATE }];
            bytes, error := encode_archive(items, format);

            assert(error.kind == .NONE);

            for n: 0..bytes.count {
                input: [] u8 = .{ n, bytes.data };
                a, e := read_archive(input, format);

                if e.kind == .NONE {
                    for a.entries {
                        data, failure := read_entry(*a, it_index);
                        free_archive(*data);
                        free_archive(*failure);
                    }
                }

                free_archive(*e);
                free_archive(*a);
            }

            for *bytes {
                old := it.*;

                it.* = 255;
                a, e := read_archive(bytes, format);

                free_archive(*e);
                free_archive(*a);
                it.* = old;
            }

            free(bytes.data);
        }

        expect(tally.outstanding == 0);
    }
} @test
