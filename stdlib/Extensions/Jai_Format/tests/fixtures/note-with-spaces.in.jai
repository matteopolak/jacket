// Spaces inside a note's parentheses stay: the note is one piece.
check :: ()   { } @skip_on(.MACOS, reason = "x y")
close :: ()   { } @tag(")  (")
