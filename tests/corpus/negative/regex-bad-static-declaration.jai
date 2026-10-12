// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

Broken :: Regex.Static("[a-");

main :: () {
    groups: Broken.Groups;
}
