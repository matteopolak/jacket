#import "Basic";

main :: () {
    value := 17;
    pointer := *value;
    result := <<pointer;

    assert(result == 17);
    print("ok\n");
}
