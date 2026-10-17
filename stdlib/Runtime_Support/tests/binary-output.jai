main :: () {
    // This runner expects byte-exact LF output. The Windows backend must avoid
    // the CRT text mode's implicit newline translation.
    write_strings("o", "k", "\n");
}
