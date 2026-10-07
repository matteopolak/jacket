// Jai_Format's public API on in-memory text, as the browser playground's Format button uses it:
// parse a config, format, refuse malformed input, and check token equivalence. Prints "ok".

#import "Basic";
#import "String";
#import "Extensions/Jai_Format";

SOURCE :: #string END
main :: ()
{
  x:=add(1,2);
  if x>2 {
  print("big\n");
  }
}
END

EXPECTED_TWO_SPACES :: #string END
main :: () {
  x := add(1, 2);
  if x > 2 {
    print("big\n");
  }
}
END

main :: () {
    // Defaults: 4 spaces, braces on the same line.
    formatted, ok, error := format_source(SOURCE);
    assert(ok, error);
    assert(begins_with(formatted, "main :: () {\n    x := add(1, 2);\n"), formatted);
    again, again_ok := format_source(formatted);
    assert(again_ok && again == formatted);
    equivalent, why := format_tokens_equivalent(SOURCE, formatted);
    assert(equivalent, why);

    config, config_ok, config_error := parse_config("indent_width = 2 # browser setting\n");
    assert(config_ok, config_error);
    two, two_ok := format_source(SOURCE, config);
    assert(two_ok && two == EXPECTED_TWO_SPACES, two);

    _, bad_ok, bad_error := parse_config("indent_width = wide\n");
    assert(!bad_ok && bad_error == "line 1: indent_width must be an integer from 1 to 16", bad_error);

    refused, refused_ok, refused_error := format_source("f :: () { x := (1; }\n");
    assert(!refused_ok && refused == "" && contains(refused_error, "unbalanced"), refused_error);

    assert(format_count_long_lines("short\nthis line is long\n", 10) == 1);
    print("ok\n");
}
