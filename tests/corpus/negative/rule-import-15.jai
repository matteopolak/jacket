// rules: import.15
// A module's #scope_file names are not reachable through its namespace.
P :: #import,string "visible :: 1;\n#scope_file\nsecret :: 2;\n";
main :: () {
    x := P.visible + P.secret;
}
