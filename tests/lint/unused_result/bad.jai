#import "Basic";
#import "String";
#import "Math";

main :: () {
    line := "  hello  ";

    trim(line);
    a := 3;

    min(a, 4);
    sqrt(2.0);
    print("%\n", line);
}

// A program's own procedure carrying the note is checked like the library's.
shout :: (s: string) -> string {
    return s;
} @UseResult

noted_user_procedure :: () {
    shout("quiet");
}
