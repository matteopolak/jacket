// Module for tests/compiler/regressions/using-beats-own-overload.jai.
#import "Using_Tie_Imported";

grab :: (size: s64, flags: s32) -> s64 {
    return 1;
}

from_file_import :: () -> s64 {
    return grab(8, 3);
}

from_body_import :: () -> s64 {
    #import "Using_Tie_Imported";
    return grab(8, 3);
}

from_using :: () -> s64 {
    Imported :: #import "Using_Tie_Imported";
    using Imported;
    return grab(8, 3);
}

from_block_using :: () -> s64 {
    {
        Imported :: #import "Using_Tie_Imported";
        using Imported;
        return grab(8, 3);
    }
}

from_nested_using :: () -> s64 {
    Imported :: #import "Using_Tie_Imported";

    inner :: () -> s64 {
        using Imported;
        return grab(8, 3);
    }

    return inner();
}
