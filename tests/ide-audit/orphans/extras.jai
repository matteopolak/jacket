Value :: struct {
    number: int;
}

constant :: Value.{ 11 };
state: Value;

helper :: (input: Value) -> int {
    return input.number + hidden;
}

#scope_file
hidden :: 7;
