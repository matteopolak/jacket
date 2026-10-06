#import "Basic";

lookup :: (key: string) -> int, bool {
    return key.count, key.count > 0;
}

declare :: () #expand {
    `made := 1;
}

main :: () {
    // Deliberately dropped.
    _, found := lookup("key");
    _unused := 3;
    print("%\n", found);
    // Declared into the caller by a macro.
    declare();
    print("%\n", made);
    // Used by inserted code.
    count := 0;
    #insert "count += 1;";
    // Used only in code for another platform.
    path := "C:\\";
    #if OS == .WINDOWS {
        print("%\n", path);
    }
}
