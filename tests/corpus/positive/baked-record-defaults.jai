Config :: struct { value: u8; extra: int = 6; }
read :: ($config: Config) -> int { return cast(int) config.value + config.extra; }
main :: () -> int {
    return read(.{value=7}) + read(Config.{extra=6, value=7});
}
