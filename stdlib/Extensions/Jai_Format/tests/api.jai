// Jai_Format's public API on in-memory text, as the browser playground's Format button uses it:
// parse a config, format, refuse malformed input, and check token equivalence. Prints "ok".

#import "Basic";
#import "String";
#import "Extensions/Jai_Format";

SOURCE :: #string JAI
main :: ()
{
  x:=add(1,2);
  if x>2 {
  print("big\n");
  }
}
JAI

EXPECTED_TWO_SPACES :: #string JAI
main :: () {
  x := add(1, 2);
  if x > 2 {
    print("big\n");
  }
}
JAI

main :: () {
    // Defaults: 4 spaces, braces on the same line.
    formatted, ok, error := format_source(SOURCE);

    assert(ok, error);
    assert(begins_with(formatted, "main :: () {\n    x := add(1, 2);\n"), formatted);
    again, again_ok := format_source(formatted);

    assert(again_ok && again == formatted);
    equivalent, why := format_tokens_equivalent(SOURCE, formatted);

    assert(equivalent, why);

    config, config_ok, config_error := parse_config("[format]\nindent_width = 2 # browser setting\n");

    assert(config_ok, config_error);
    two, two_ok := format_source(SOURCE, config);

    assert(two_ok && two == EXPECTED_TWO_SPACES, two);

    _, bad_ok, bad_error := parse_config("[format]\nindent_width = wide\n");

    assert(!bad_ok && bad_error == "line 2: indent_width must be an integer from 1 to 16", bad_error);

    refused, refused_ok, refused_error := format_source("f :: () { x := (1; }\n");

    assert(!refused_ok && refused == "" && contains(refused_error, "unbalanced"), refused_error);

    assert(format_count_long_lines("short\nthis line is long\n", 10) == 1);

    // The nearest jacket.toml above a directory, found through the caller's file reader.
    found, file, found_ok := format_find_config("/proj/src/deep", memory_files);

    assert(found_ok && file == "/proj/jacket.toml" && found.indent_width == 3 && found.root == "/proj");

    _, stopped_file, stopped_ok := format_find_config("/proj/src/deep", memory_files, "/proj/src");

    assert(stopped_ok && stopped_file == "");

    _, broken_file, broken_ok, broken_error := format_find_config("/bad/x", memory_files);

    assert(!broken_ok && broken_file == "/bad/jacket.toml" && begins_with(broken_error, "line 2:"), broken_error);
    assert(format_parent_directory("/a/b/c") == "/a/b" && format_parent_directory("/a") == "/" && format_parent_directory("a") == "");
    print("ok\n");
}

memory_files :: (path: string) -> string, bool {
    if path == "/proj/jacket.toml" return "[format]\nindent_width = 3\n", true;
    if path == "/bad/jacket.toml" return "[format]\nindent_width = x\n", true;

    return "", false;
}
