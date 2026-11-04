#import "Basic";

main :: () {
    pointers: [4] *int;
    pointers[1] = New(int);
    pointers[2] = New(int);
    pointers[3] = New(int);

    defer {
        for pointers free(it);
    }
}

same_element :: () {
    pointers: [2] *int;
    pointers[0] = New(int);
    pointers[1] = New(int);
    for pointers {
        free(it);
        free(it);
    }
}
