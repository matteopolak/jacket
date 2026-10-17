#assert OS_IS_UNIX || OS == .WINDOWS;

// The module is the files of internal/net loaded into one scope: the per-OS native declarations
// (socket-abi-<os>.jai) are the public names, beside the hand-written helpers around them.
#load "../internal/net/socket.jai";
