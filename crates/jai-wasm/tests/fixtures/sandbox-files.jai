// Private SandboxHost stdio protocol; source and created files live in the in-memory filesystem.
fopen :: (path: *u8, mode: *u8) -> *void #foreign;
fwrite :: (data: *void, size: u64, count: u64, file: *void) -> u64 #foreign;
fread :: (data: *void, size: u64, count: u64, file: *void) -> u64 #foreign;
fclose :: (file: *void) -> s32 #foreign;
write :: (descriptor: s32, text: *u8, count: u64) -> s64 #foreign;

main :: () -> s32 {
    file := fopen("/tmp/x.txt\0".data, "wb\0".data);
    if file == null return 1;
    if fwrite("data".data, 1, 4, file) != 4 return 2;
    if fclose(file) != 0 return 3;

    file = fopen("/tmp/x.txt\0".data, "rb\0".data);
    if file == null return 4;

    bytes: [16] u8;
    if fread(bytes.data, 1, 16, file) != 4 return 5;
    if bytes[0] != 100 || bytes[1] != 97 || bytes[2] != 116 || bytes[3] != 97 return 6;
    if fread(bytes.data, 1, 1, file) != 0 return 7;
    if fclose(file) != 0 return 8;

    own := fopen("main.jai\0".data, "rb\0".data);
    if own == null return 9;
    if fread(bytes.data, 1, 16, own) != 16 return 10;
    if bytes[0] != 47 || bytes[1] != 47 return 14;
    if fclose(own) != 0 return 11;

    missing := fopen("not-created.jai\0".data, "rb\0".data);
    if missing != null return 12;
    if write(1, "ok\n".data, 3) != 3 return 13;

    return 0;
}
