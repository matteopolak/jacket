#import "Basic";

Config :: struct {
    width: int = 80;
    height: int = 24;
    title: string;
}

make :: (title: string) -> Config {
    config: Config;
    config.width = 100;
    config.title = title;

    return config;
}

main :: () {
    print("%\n", make("demo"));
}
