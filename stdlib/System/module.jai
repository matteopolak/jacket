// Asks the operating system about the machine, the user and the running program.
//
// - [get_username], [get_home_directory] and [get_machinename] for who and where.
// - [get_path_of_running_executable] for the program's own path.
// - [get_number_of_processors] for how many cores there are.
// - [get_error_value] and [get_error_string] for the last OS error.
//
// Strings returned here live in temporary storage; copy one to keep it.
//
// ```jai
// path := get_path_of_running_executable();
// assert(path.count > 0);
// ```

// test: skip_on(.WASM, reason = "System has no wasm target")

#assert OS == .WINDOWS || OS == .MACOS || OS == .LINUX || OS == .ANDROID;
// Which processors [get_number_of_processors] counts: physical cores or logical threads, all of them or only the performance ones on hybrid chips.
Processor_Query_Type :: Impl.Processor_Query_Type;
// Returns a fast, monotonic tick count for timing short spans. Ticks are not seconds and the rate varies by machine, so compare readings only with each other.
//
// ```jai
// start := read_cpu_counter();
// stop := read_cpu_counter();
// assert(stop >= start);
// ```
read_cpu_counter :: Impl.read_cpu_counter;

#if CPU == .X64 M64 :: #import "Machine_X64";

// Returns the name of the current user and whether the lookup worked.
//
// ```jai
// name, ok := get_username();
// assert(!ok || name.count > 0);
// ```
get_username :: Impl.get_username;
// Returns the home directory of `username`, or of the current user when it is empty, and whether the lookup worked.
//
// ```jai
// home, ok := get_home_directory();
// assert(!ok || home.count > 0);
// ```
get_home_directory :: Impl.get_home_directory;
// Returns the machine's host name, or an empty string if it cannot be read.
//
// ```jai
// assert(get_machinename().count > 0);
// ```
get_machinename :: Impl.get_machinename;
// Returns the full path of the running program, or an empty string if it cannot be found.
//
// ```jai
// assert(get_path_of_running_executable().count > 0);
// ```
get_path_of_running_executable :: Impl.get_path_of_running_executable;
// How many processors of a kind the machine has.
//
// ```jai
// logical := get_number_of_processors(.ALL_LOGICAL);
// assert(logical >= get_number_of_processors(.ALL_PHYSICAL) && logical > 0);
// ```
get_number_of_processors :: Impl.get_number_of_processors;

#if OS == .WINDOWS {
    Windows :: #import "Windows";

    // Returns the last error code the operating system set: `errno` on Unix, `GetLastError` on Windows. Read it right after the call that failed.
    //
    // ```jai
    // description := get_error_string(get_error_value());
    // assert(description.count >= 0);
    // ```
    get_error_value :: Windows.GetLastError;
} else {
    // Returns the last error code the operating system set: `errno` on Unix, `GetLastError` on Windows. Read it right after the call that failed.
    get_error_value :: P.errno;
}

// Returns the system's message for an error code, such as "No such file or directory".
//
// ```jai
// assert(get_error_string(get_error_value()).count >= 0);
// ```
get_error_string :: Impl.get_error_string;

// Returns the last OS error code together with its message.
//
// ```jai
// code, message := get_error_value_and_string();
// assert(message.count >= 0);
// ```
get_error_value_and_string :: Impl.get_error_value_and_string;

// Returns the user database entry for `username`, or for the current user when it is empty. Linux and Android only.
#if OS == .LINUX || OS == .ANDROID get_passwd_for_user :: Impl.get_passwd_for_user;

#scope_file
#if OS == .WINDOWS Impl :: #import "internal/system/windows";

else {
    P :: #import "POSIX";
    Impl :: #import "internal/system/unix";
}
