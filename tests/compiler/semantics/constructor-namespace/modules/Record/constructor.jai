create :: () -> *Buffer {
    return F.construct(Buffer, *storage);
}

#scope_module
storage: Buffer;

#scope_file
F :: #import "Factory";
