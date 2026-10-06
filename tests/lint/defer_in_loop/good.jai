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
    // The loop's own step, deferred so that `continue` takes it too.
    i := 0;
    while i < names.count {
        defer i += 1;
        if names[i] == "a" continue;
        print("%\n", names[i]);
    }
}
