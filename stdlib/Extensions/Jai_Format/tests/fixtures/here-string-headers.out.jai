// A here-string's terminator name may start a later line, after blank lines or comments.
main :: () {
    a := #string
        END
body
END;
    b := #string // a note
    /* a /* nested */ block */
        TEXT
more
TEXT;
}
