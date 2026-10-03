Pair :: struct { value: int; }
convert :: (input: Pair) #expand {
    result := input;
    defer result.value = 99;
    `return result;
}
answer :: () -> Pair {
    result := 100;
    input := Pair.{42};
    convert(input);
}
main :: () -> int {
    result := answer();
    return result.value;
}
