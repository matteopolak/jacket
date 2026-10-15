// Reviewed public map/signatures; bit meanings and reserved blocks from official LLVM MSF docs.
#import "Basic";
#import "File";
D :: #import "Debug_Info";

main :: () {
    for architecture: string.["x64", "arm64"] {
        bytes, ok := read_entire_file(tprint("%../../target/developer-tools-msf-%.pdb", #filepath, architecture));

        assert(ok);
        defer free(bytes);
        oracle, oracle_ok := read_entire_file(tprint("%../../target/developer-tools-msf-%.fpm.bin", #filepath, architecture));

        assert(oracle_ok && oracle.count == 4096);
        defer free(oracle);
        msf: D.Msf;
        msf.header = cast(type_of(msf.header)) bytes.data;
        map: D.Free_Page_Map;
        map.cbPg = 4096;
        map.fBigMsf = true;
        map.iwMac = 4096 / 8;
        array_resize(*map.rgw, map.iwMac);
        defer array_free(map.rgw);
        start := cast(s64) msf.header.data.valid_fpm_page_no * 4096;

        for word: 0..map.rgw.count - 1 {
            value: u64;

            for byte: 0..7 {
                wide := cast(u64) bytes[start + word * 8 + byte];
                value |= wide << (byte * 8);
            }

            map.rgw[word] = cast(s64) value;
        }

        free_count: s64;
        used_count: s64;

        for page: 0..32767 {
            expected := (oracle[page / 8] & (cast(u8) 1 << (page % 8))) != 0;
            assert(D.is_free_page(map, cast(u32) page) == expected);
            if expected free_count += 1;
            else used_count += 1;
        }

        assert(free_count > 0 && used_count > 0);
        assert(!D.is_free_page(map, 0) && !D.is_free_page(map, 1) && !D.is_free_page(map, 2));

        // Exercise signed-word bit63 and boundaries without changing borrowed native bytes.
        for page: u32.[0, 1, 2, 63, 64, 127, 128, 32767] {
            original := D.is_free_page(map, page);

            D.alloc_page(*map, page);
            assert(!D.is_free_page(map, page));
            D.alloc_page(*map, page);
            assert(!D.is_free_page(map, page));
            D.free_page(*map, page);
            assert(D.is_free_page(map, page));
            D.free_page(*map, page);
            assert(D.is_free_page(map, page));
            if !original D.alloc_page(*map, page);
        }

        for word: 0..map.rgw.count - 1 {
            value := cast(u64) map.rgw[word];
            for byte: 0..7 assert(cast(u8) ((value >> (byte * 8)) & 0xff) == bytes[start + word * 8 + byte]);
        }

        for page: u32.[0, 1, 2, 3, 4096, 4097, 4098, 4099, 8193, 8194, 0xffffffff] {
            expected := page % 4096 == 1 || page % 4096 == 2;
            assert(D.is_fpm_page_number(map, page) == expected);
        }

        assert(map.iwRover == 0 && map.wFill == 0 && map.iwMac == 512);
    }

    print("ok\n");
}
