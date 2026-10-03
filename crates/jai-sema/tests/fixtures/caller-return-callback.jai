Required :: #type (value: int) -> int #must;
callback :: (value: int) -> int { return value + 2; }
finish :: () #expand { `return callback; }
answer :: () -> Required {
    Required :: #type (different: int) -> int;
    finish();
}
main :: () -> int { result := answer(); return result(value = 40); }
