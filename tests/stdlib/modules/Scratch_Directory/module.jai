// A directory for the files a test writes, under the system's temporary directory (TMPDIR on
// Unix, TEMP on Windows, /tmp when unset). Each process gets its own, so tests running at the
// same time never share files: `<temporary>/jaic-<name>-<process id>`. The caller deletes it
// (`delete_directory` from File) when done.
scratch_directory :: (name: string) -> string {
    base := environment_value(ifx OS == .WINDOWS then "TEMP" else "TMPDIR");
    if base.count == 0 base = "/tmp";
    while base.count > 1 && (base[base.count - 1] == #char "/" || base[base.count - 1] == #char "\\") base.count -= 1;
    dir := tprint("%/jaic-%-%", base, name, process_id());
    assert(make_directory_if_it_does_not_exist(dir, recursive = true), "cannot create the scratch directory %", dir);
    return dir;
}

#scope_file
#import "Basic";
#import "File";

#if OS == .WINDOWS {
    Windows :: #import "Windows";
    environment_value :: (name: string) -> string {
        value := Windows.getenv(temp_c_string(name));
        return ifx value then to_string(value) else "";
    }
    process_id :: () -> u32 {
        return Windows.GetCurrentProcessId();
    }
} else {
    POSIX :: #import "POSIX";
    environment_value :: (name: string) -> string {
        value := POSIX.getenv(temp_c_string(name));
        return ifx value then to_string(value) else "";
    }
    process_id :: () -> s64 {
        return POSIX.getpid();
    }
}
