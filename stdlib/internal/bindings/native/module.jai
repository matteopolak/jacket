// Private native buffer operations used by explicit public wrappers.
terminated_copy :: (text: string) -> *u8 #no_context {
    p := cast(*u8) malloc(cast(u64) text.count + 1);
    if !p abort();
    for i: 0..text.count - 1 p[i] = text[i];

    p[text.count] = 0;

    return p;
}

release :: (memory: *void) #no_context {
    free(memory);
} @Releases

allocate :: (bytes: s64) -> *void #no_context {
    p := malloc(cast(u64) bytes);
    if !p && bytes != 0 abort();
    return p;
}

#scope_file
libc :: #library,system "c";
malloc :: (bytes: u64) -> *void #foreign libc;
free :: (memory: *void) #foreign libc;
abort :: () #foreign libc;
