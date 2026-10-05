#import "Basic";
#import "String";

// scan2 pattern matching, path_strip_extension, is_absolute_path, CRLF normalization and
// the parse_* token readers.

Color :: enum { RED; GREEN; BLUE; }

main :: () -> int {
    a, b: int;
    f: float;
    if !scan2("(1 2)", "(% %)", *a, *b) || a != 1 || b != 2  return 1;
    if !scan2("x=3.25;", "x=%;", *f) || f != 3.25  return 2;
    if !scan2("%5", "%%%", *a) || a != 5  return 3;
    if scan2("1 2 3", "% %", *a, *b)  return 4;          // text left over
    if scan2("1 2", "% %", *a)  return 5;                // too few arguments
    if scan2("1 2", "% %", a, *b)  return 6;             // not a pointer
    if scan2("abc", "abd") || !scan2("abc", "abc") || scan2("ab", "abc")  return 7;

    if path_strip_extension("a/b.c") != "a/b" || path_strip_extension("x.tar.gz") != "x.tar"  return 10;
    if path_strip_extension("a.b/c") != "a.b/c" || path_strip_extension(".hidden") != ".hidden"  return 11;
    #if OS != .WINDOWS {
        if !is_absolute_path("/abs") || is_absolute_path("rel") || is_absolute_path("")  return 12;
    }

    text := copy_string("a\r\nb\r\r\nc\r");
    normalize_line_endings(*text);
    if text != "a\nb\r\nc\r"  return 13;

    line := " GREEN purple 3.5 x 42";
    color, ok := parse_enum(*line, Color);
    if !ok || color != .GREEN  return 20;
    color, ok = parse_enum(*line, Color);
    if ok || color != .RED  return 21;
    number, fok := parse_float(*line);
    if !fok || number != 3.5  return 22;
    number, fok = parse_float(*line);
    if fok  return 23;
    small, iok := parse_int(*line, s8);
    if !iok || small != 42  return 24;
    small, iok = parse_int(*line, s8);
    if iok || small != 0  return 25;
    return 0;
}
