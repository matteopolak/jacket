/* A block comment can contain apparent code.
fake_comment_declaration :: () {}
*/
banner :: #string END_BANNER
fake_here_string_declaration :: () {}
END_BANNER
multiline :: "first line
second line";
Mode :: enum {
    FIRST;
    SECOND :: 7;
    THIRD;
}
#if true {
    enabled :: 1;
} else {
    disabled :: 2;
}
main :: () {
    word := "😀漢é";
    healthy := 42;
    result := healthy + 1;
}
