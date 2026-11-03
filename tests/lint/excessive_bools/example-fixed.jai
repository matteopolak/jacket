#import "Basic";

Open_Flags :: enum_flags u8 {
    READ;
    WRITE;
    CREATE;
    TRUNCATE;
}

open :: (path: string, flags: Open_Flags) -> int {
    return path.count;
}

main :: () {
    print("%\n", open("a.txt", .READ | .CREATE));
}
