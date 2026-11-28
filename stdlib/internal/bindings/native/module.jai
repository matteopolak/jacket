// Private native buffer operations used by explicit public wrappers.
terminated_copy :: (text: string) -> *u8 #no_context {
    p := cast(*u8) c_malloc(cast(u64) text.count + 1);
    if !p c_abort();
    for i: 0..text.count - 1 p[i] = text[i];

    p[text.count] = 0;

    return p;
}

release :: (memory: *void) #no_context {
    c_free(memory);
}

allocate :: (bytes: s64) -> *void #no_context {
    p := c_malloc(cast(u64) bytes);
    if !p && bytes != 0 c_abort();
    return p;
}

#scope_file
#import "internal/libc";
