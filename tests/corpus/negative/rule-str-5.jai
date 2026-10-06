// rules: str.5
// An unknown backslash escape in a string literal is a lexer error.
main :: () {
    s := "bad \q escape";
}
