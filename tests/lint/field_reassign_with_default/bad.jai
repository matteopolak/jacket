#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
    verbose: bool;
}

make :: (title: string) -> Config {
    config: Config;
    config.width = 100;
    config.title = title;

    return config;
}

single :: () -> int {
    other: Config;
    other.height = 5;

    return other.height;
}

main :: () {
    c := make("x");
    print("% %\n", c, single());
}
