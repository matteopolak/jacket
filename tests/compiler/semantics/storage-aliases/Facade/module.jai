using,only(Handle, Pair, storage_alias_counter, storage_alias_pair, storage_alias_read, storage_alias_write) #import "Backing";

default_read :: (value: Handle = storage_alias_counter) -> Handle #no_context {
    return value;
}

classify :: (value: $T) -> bool #no_context {
    return T == Handle && type_of(value) == Handle;
}
