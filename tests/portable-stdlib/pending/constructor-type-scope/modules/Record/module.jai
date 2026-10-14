Buffer :: struct {
    value: s64;
}

create :: () -> *Buffer {
    return New(Buffer);
}

#scope_file
#import "Basic";
