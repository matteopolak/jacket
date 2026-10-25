// The process environment.

// The value of the environment variable `name`, in temporary storage, or "" when it is not set.
// On Windows the name and value go through UTF-16, so non-ASCII values come back as UTF-8.
environment_variable :: (name: string) -> string {
    value, _ := read_variable(name);
    return value;
}

// Whether `name` is set at all, which `environment_variable` does not say for a variable set to
// the empty string.
has_environment_variable :: (name: string) -> bool {
    _, set := read_variable(name);
    return set;
}

#scope_file

#import "Basic";

#if OS == .WINDOWS {
    Windows :: #import "Windows";
    #import "Windows_Utf8";
} else {
    libc :: #system_library "libc";
    c_getenv :: (name: *u8) -> *u8 #foreign libc "getenv";
}

read_variable :: (name: string) -> value: string, set: bool {
    #if OS == .WINDOWS {
        wide_name, ok := utf8_to_wide(name,, temp);
        if !ok return "", false;

        wide_value := Windows._wgetenv(cast(*s16) wide_name);
        if !wide_value return "", false;

        value, converted := wide_to_utf8(cast(*u16) wide_value,, temp);

        return value, converted;
    } else {
        value := c_getenv(temp_c_string(name));
        if !value return "", false;

        return copy_string(to_string(value),, temp), true;
    }
}
