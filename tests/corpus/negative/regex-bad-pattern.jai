// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

main :: () {
    Regex.is_match(Regex.Static("ab(c"), "abc");
}
