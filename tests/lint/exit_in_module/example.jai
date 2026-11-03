#import "Basic";

open_config :: (path: string) -> int {
    if path.count == 0 {
        exit(1);
    }

    return path.count;
}
