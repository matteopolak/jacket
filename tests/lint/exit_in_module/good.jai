#import "Basic";

open_config :: (path: string) -> (int, bool) {
    if path.count == 0 return 0, false;

    return path.count, true;
}

// Not the library's `exit`.
exit_code :: (ok: bool) -> s32 {
    if ok return 0;
    return 1;
}

main_status :: (ok: bool) -> s32 {
    return exit_code(ok);
}
