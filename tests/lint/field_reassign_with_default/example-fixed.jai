#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
}

make :: (title: string) -> Config {
    return .{ width = 100, title = title };
}

main :: () {
    print("%\n", make("demo"));
}
