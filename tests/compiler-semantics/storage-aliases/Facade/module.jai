using,only(Handle, Pair, cleanroom_alias_counter, cleanroom_alias_pair, cleanroom_alias_read, cleanroom_alias_write) #import "Backing";

default_read :: (value: Handle = cleanroom_alias_counter) -> Handle #no_context {
    return value;
}

classify :: (value: $T) -> bool #no_context {
    return T == Handle && type_of(value) == Handle;
}
