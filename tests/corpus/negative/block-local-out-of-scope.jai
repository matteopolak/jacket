// A local ends with the block that declares it.
#import "Basic";
main :: () {
    {
        inner := 5;
    }
    print("%\n", inner);
}
