#scope_module

// Eight 256-entry tables, built at compile time: table k advances a byte through k further zero bytes.
CRC_TABLES :: #run make_crc_tables();

make_crc_tables :: () -> [2048] u32 {
    tables: [2048] u32;

    for i: 0..255 {
        c := cast(u32) i;

        for 0..7 {
            if c & 1 c = (c >> 1) ^ 0xEDB88320;
            else c >>= 1;
        }

        tables[i] = c;
    }

    for k: 1..7 {
        for i: 0..255 {
            previous := tables[(k - 1) * 256 + i];
            tables[k * 256 + i] = (previous >> 8) ^ tables[previous & 255];
        }
    }

    return tables;
}

crc32_update :: (data: [] u8, crc: u32) -> u32 #no_abc {
    c := ~crc;
    p := data.data;
    n := data.count;
    i: s64;

    // Indices below are masked or shifted to 0..255 plus a multiple of 256 under 2048.
    while i + 8 <= n {
        a := (cast(*u32) (p + i)).* ^ c;
        b := (cast(*u32) (p + i + 4)).*;

        c = CRC_TABLES[1792 + (a & 255)] ^ CRC_TABLES[1536 + ((a >> 8) & 255)] ^ CRC_TABLES[1280 + ((a >> 16) & 255)] ^ CRC_TABLES[1024 + (a >> 24)]
          ^ CRC_TABLES[768 + (b & 255)] ^ CRC_TABLES[512 + ((b >> 8) & 255)] ^ CRC_TABLES[256 + ((b >> 16) & 255)] ^ CRC_TABLES[b >> 24];
        i += 8;
    }

    while i < n {
        c = CRC_TABLES[(c ^ p[i]) & 255] ^ (c >> 8);
        i += 1;
    }

    return ~c;
}

// Sums stay below 2^32 for blocks of up to 5552 bytes, so the modulo runs once per block.
adler32_update :: (data: [] u8, adler: u32) -> u32 #no_abc {
    a: u32 = adler & 0xFFFF;
    b: u32 = adler >> 16;
    p := data.data;
    n := data.count;
    i: s64;

    while i < n {
        end := min(i + 5552, n);

        // 32 bytes at a time without a serial chain: b gains 32 * a plus each byte weighted by
        // its distance from the end of the group.
        while i + 32 <= end {
            sum: u32;
            weighted: u32;

            for k: 0..31 {
                v: u32 = p[i + k];
                sum += v;
                weighted += v * cast(u32) (32 - k);
            }

            b += 32 * a + weighted;
            a += sum;
            i += 32;
        }

        while i < end {
            a += p[i];
            b += a;
            i += 1;
        }

        a %= 65521;
        b %= 65521;
    }

    return (b << 16) | a;
}
