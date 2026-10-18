// Big constant tables: arrays computed at compile time and baked into the program.
#import "Basic";

CRC_TABLE :: #run -> [256] u32 {
    table: [256] u32;

    for 0..255 {
        c := cast(u32) it;
        for 1..8 c = ifx c & 1 then 0xEDB88320 ^ (c >> 1) else c >> 1;
        table[it] = c;
    }

    return table;
};

SIZE :: 400;

PRODUCTS :: #run -> [SIZE * SIZE] u32 {
    table: [SIZE * SIZE] u32;

    for row: 0..SIZE - 1 {
        for column: 0..SIZE - 1 {
            table[row * SIZE + column] = cast(u32) (row * column) ^ CRC_TABLE[(row + column) & 255];
        }
    }

    return table;
};

SQUARES :: #run -> [16384] s32 {
    table: [16384] s32;
    for 0..16383 table[it] = cast(s32) ((it * it) % 1000003);
    return table;
};

main :: () {
    checksum: u32;
    for 0..SIZE * SIZE - 1 checksum ^= PRODUCTS[it] + cast(u32) SQUARES[it & 16383];
    print("%\n", checksum);
}
