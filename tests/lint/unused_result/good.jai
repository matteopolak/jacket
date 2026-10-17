#import "Basic";
#import "Math";
Str :: #import "String";

// A program's own `trim` that works in place.
trim :: (s: *string) {
    s.* = Str.trim(s.*);
}

main :: () {
    line := "  hello  ";
    line = Str.trim(line);
    v := Vector3.{ 3, 4, 0 };

    normalize(*v);
    other := " x ";

    trim(*other);
    print("% % %\n", line, v, other);
}

// Without the note the name means nothing, even a library's name.
trim :: (s: string) -> string {
    return s;
}

plain :: (s: string) -> string {
    return s;
}

unnoted_user_procedures :: () {
    trim("quiet");
    plain("quiet");
}
