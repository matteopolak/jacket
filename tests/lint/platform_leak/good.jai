// Held to every system by the test settings; the import is only there where POSIX builds.
#if OS == .LINUX || OS == .MACOS {
    P :: #import "POSIX";
}

main :: () {
    #if OS == .LINUX || OS == .MACOS {
        P.getpid();
    }
}
