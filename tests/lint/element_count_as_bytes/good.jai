#import "Basic";

copy_values :: (dest: [] s32, values: [] s32) {
    memcpy(dest.data, values.data, values.count * size_of(s32));
}

copy_bytes :: (dest: [] u8, source: [] u8) {
    memcpy(dest.data, source.data, source.count);
}

copy_text :: (dest: *u8, text: string) {
    memcpy(dest, text.data, text.count);
}

clear_flags :: (flags: [] bool) {
    memset(flags.data, 0, flags.count);
}

main :: () {
    a: [4] s32;
    b: [4] s32;

    copy_values(a, b);
    x: [4] u8;

    copy_bytes(x, x);
    copy_text(x.data, "ab");
    flags: [2] bool;

    clear_flags(flags);
}
