#import "Basic";

open_config :: (path: string) -> int, bool {
    if path.count == 0 return 0, false;

    return path.count, true;
}
