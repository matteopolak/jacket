Buffer :: struct {
    value: int = seed;
    next: *Buffer;
}

#scope_file
seed :: 37;
