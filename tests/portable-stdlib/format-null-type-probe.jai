// Official native observation: "void\n{void, 0}\n" for these defaults.
// Current compiler initializes them incorrectly; never mask this in formatting.
#import "Basic";

Record :: struct {
    descriptor: Type;
    count: int;
}

main :: () {
    descriptor: Type;
    record: Record;
    print("%\n", descriptor);
    print("%\n", record);
}
