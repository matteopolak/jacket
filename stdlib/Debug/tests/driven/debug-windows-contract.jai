#if OS == .WINDOWS {
    #import "Basic";
    D :: #import "Debug";

    compile_contract :: () {
        presence: #type () -> bool = D.os_is_debugger_present;
        capture: #type (trace: [] *void, maxcount: int, skip: int) -> int = D.os_backtrace;
        symbols: #type (trace: [] *void, n: int) -> [] string = D.os_backtrace_symbols;
        probe: #type (ptr: *void) -> bool = D.is_valid_pointer;

        assert(presence != null && capture != null && symbols != null && probe != null);
        assert(D.backtrace_symbols == D.os_backtrace_symbols);
        assert(D.translate_symbols == D.backtrace_symbols);
        initialize: #type () = D.init;
        enable: #type () = D.os_enable_signal_handler;
        disable: #type () = D.os_disable_signal_handler;
        logger: #type (message: string, data: *void, info: Log_Info) = D.os_logger;

        assert(initialize != null && enable != null && disable != null && logger != null);
    }
}
