#import "Basic";

read :: (value: Any) -> float32 {
    pointer := cast(*float32) value.value_pointer;
    return pointer.*;
}

local_holder :: () -> float32 {
    value: Any = 3.5;
    return read(value);
}

field_holder :: () -> float32 {
    Holder :: struct {
        value: Any;
    }

    holder: Holder;
    holder.value = 3.5;

    return read(holder.value);
}

returned_holder :: () -> Any {
    value: Any = 3.5;
    return value;
}

saved: Any;

bad_global :: () {
    saved = 3.5;
}
