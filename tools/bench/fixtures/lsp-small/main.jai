// A complete small program for useful LSP navigation measurements, with no module imports.
twice :: (value: int) -> int {
    result := value;
    result += value;
    return result;
}

main :: () {
    total := twice(21);
    total += twice(total);
}
