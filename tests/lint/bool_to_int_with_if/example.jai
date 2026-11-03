#import "Basic";

main :: () {
    ready := true;
    count := ifx ready then 1 else 0;
    print("%\n", count);
}
