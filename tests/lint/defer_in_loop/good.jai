#import "Basic";

main :: () {
    names := string.["a", "b"];
    // Taken and released in the same iteration.
    for names {
        copy := copy_string(it);
        defer free(copy);
        print("%\n", copy);
    }
    // Outside the loop: runs once, at the end of `main`.
    buffer := alloc(64);
    defer free(buffer);
    // Paired with a change earlier in the same iteration.
    depth := 0;
    for names {
        depth += 1;
        defer depth -= 1;
        print("% %\n", depth, it);
    }
}
