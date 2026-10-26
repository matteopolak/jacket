// A project's own Windows that declares only what the project uses.
#assert OS == .WINDOWS, "this fixture is a Windows project";

DWORD :: u32;

GetCurrentProcessId :: () -> DWORD #foreign kernel32;
GetLastError :: () -> DWORD #foreign kernel32;

#scope_file
kernel32 :: #library,system "kernel32";
