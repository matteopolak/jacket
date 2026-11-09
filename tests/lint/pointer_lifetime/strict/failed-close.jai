W :: #import "Windows";

// CloseHandle gives the handle up even when it returns zero, so closing it again is a second release.
failed_close :: (handle: *void) {
    if W.CloseHandle(handle) != 0 return;
    if W.GetThreadId(handle) == 0 return;
    if W.CloseHandle(handle) != 0 return;
}

main :: () {}
