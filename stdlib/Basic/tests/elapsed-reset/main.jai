#import "Basic";

// This compile-time/native pairing uses the worker's actual macOS clock ABI.
#if OS == .MACOS {
    #run {
        assert(seconds_since_init() == 0);
        assert(seconds_since_init() >= 0);
    }

    main :: () {
        assert(seconds_since_init() == 0, "compile-time elapsed origin must not survive target initialization");
        print("ok\n");
    }
} else {
    main :: () {
        assert(false, "elapsed reset pairing requires actual macOS execution");
    }
}
