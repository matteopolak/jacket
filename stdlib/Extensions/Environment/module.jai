// Environment: reading the variables the program was started with.
//
// The values are text in the environment as the system hands it over; this module only reads. Setting variables is not
// part of it.
//
// ```jai,skip_on(wasm)
// assert(has_environment_variable("PATH"));
// assert(!has_environment_variable("JAI_DOC_SURELY_UNSET_VARIABLE"));
// ```

// The value of the environment variable `name`, in temporary storage, or "" when it is not set.
// On Windows the name and value go through UTF-16, so non-ASCII values come back as UTF-8.
//
// The string is valid until the temporary storage is reset: use `copy_string` to keep it. A name that is not set,
// and one that cannot be looked up (an empty name or, on Windows, text that is not valid UTF-8), give "" as well; use
// `has_environment_variable` to tell an empty value from an unset variable.
//
// ```jai,skip_on(wasm)
// assert(environment_variable("PATH").count > 0); // set on every desktop platform
// assert(environment_variable("JAI_DOC_SURELY_UNSET_VARIABLE") == "");
//
// kept := copy_string(environment_variable("PATH"));
// defer free(kept);
// assert(kept.count > 0);
// ```
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
