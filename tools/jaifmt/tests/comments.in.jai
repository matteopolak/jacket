// Header comment.
/* A block
   comment at the top. */

f :: () {
        /* An indented
           block comment moves
           with its first line. */
    x := 1;    // aligned trailing comments
    yy := 22;  // stay aligned
    z := 3;// gets a space
    /* nested /* block */ comment */ w := 4;
// a comment at column zero is reindented
    return;
}
