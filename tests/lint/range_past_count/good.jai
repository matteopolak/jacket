#import "Basic";

main :: () {
    names := string.["a", "b", "c"];
    for i: 0..names.count - 1 {
        print("%\n", names[i]);
    }
    for i: 0..names.count {
        if i == names.count break;
        print("%\n", names[i]);
    }
    for i: 0..names.count print("% ", i); // no indexing
}
