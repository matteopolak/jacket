#import "Basic";
main :: () {
    pointers: [4] *int;
    pointers[1] = New(int);
    pointers[2] = New(int);
    pointers[3] = New(int);
    defer { for pointers free(it); }
}
