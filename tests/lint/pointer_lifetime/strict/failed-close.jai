W :: #import "Windows";

// A protected handle remains usable after CloseHandle returns zero.
failed_close :: (handle: *void) {
    if W.CloseHandle(handle) != 0 return;
    if W.GetThreadId(handle) == 0 return;
    if W.CloseHandle(handle) != 0 return;
}

main :: () {}
