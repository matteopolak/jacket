Number :: struct { value: u8; extra: int = 6; }
Flag :: struct { value: bool; }
choose :: (item: Number) -> int { return cast(int) item.value + item.extra; }
choose :: (item: Flag) -> int { return 42; }
main :: () -> int { return choose(.{value=7}); }
