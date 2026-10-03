// Independent access to the SDK thread-local errno slot.
errno :: () -> OS_Error_Code #no_context {
    slot := __errno_location();
    return slot.*;
}

set_errno :: (value: OS_Error_Code) #no_context {
    slot := __errno_location();
    slot.* = value;
}
