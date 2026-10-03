apply :: () #expand {
    argument := 1;
    `result := `callback(argument);
}

main :: () -> int {
    callback :: (value: int) -> int { return value + 41; }
    argument := 100;
    apply();
    return result + argument - 100;
}
