// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

main :: () {
    Regex.parse(int, "1");
}
