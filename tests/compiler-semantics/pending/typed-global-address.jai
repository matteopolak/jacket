#import "Basic";

Settings :: struct {
    callback := read_count;
    count: s64;
    owner: *Settings = *settings;
}

settings: Settings;

read_count :: () -> s64 {
    return settings.count;
}

main :: () {
    assert(settings.owner == *settings);
    assert(settings.callback() == 0);
    settings.count = 42;
    assert(settings.callback() == 42);
    print("ok\n");
}
